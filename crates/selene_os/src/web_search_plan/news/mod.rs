#![forbid(unsafe_code)]

pub mod brave_news;
pub mod conflict;
pub mod diversity;
pub mod gdelt;
pub mod merge;
pub mod recency;

use crate::web_search_plan::cache::cache_key::{CacheKey, CacheMode};
use crate::web_search_plan::cache::l1::L1Cache;
use crate::web_search_plan::cache::ttl::ttl_ms_for;
use crate::web_search_plan::cache::{lookup_typed, store_typed, CacheLookupHit};
use crate::web_search_plan::chunk::bounded_excerpt;
use crate::web_search_plan::chunk::chunker::{TextChunk, CHUNK_VERSION};
use crate::web_search_plan::chunk::hasher::{derive_chunk_id, Sha256ChunkHasher, HASH_VERSION};
use crate::web_search_plan::chunk::normalize::{normalize_document_for_chunking, NORM_VERSION};
use crate::web_search_plan::diag::{
    default_degraded_transitions, default_failed_transitions, try_build_debug_packet,
    DebugPacketContext, DebugStatus, HealthStatusBeforeFallback,
};
use crate::web_search_plan::news::conflict::{
    build_conflict_clusters, cluster_lookup_by_canonical_url,
};
use crate::web_search_plan::news::diversity::{distinct_domain_count, diversity_threshold_met};
use crate::web_search_plan::news::merge::merge_news_results;
use crate::web_search_plan::news::recency::{
    freshness_score, normalize_published_at, recency_window_days, within_recency_window,
    ImportanceTier,
};
use crate::web_search_plan::perf_cost::budgets::ProviderCallBudget;
use crate::web_search_plan::perf_cost::tiers::{
    caps_for_tier, ImportanceTier as PerfImportanceTier,
};
use crate::web_search_plan::perf_cost::timeouts::clamp_provider_timeout;
use crate::web_search_plan::proxy::proxy_config::{ProxyConfig, SystemEnvProvider};
use crate::web_search_plan::proxy::ProxyMode;
use crate::web_search_plan::web_provider::health_state::{
    HealthPolicy, ProviderHealthState, ProviderHealthTracker,
};
use selene_engines::ph1providerctl::{
    disabled_provider_decision, fake_provider_decision, is_local_fake_endpoint,
    provider_fallback_enabled_from_env, ProviderControlProvider, ProviderControlRoute,
    ProviderGateDecision,
};
use selene_kernel_contracts::provider_secrets::ProviderSecretId;
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};
use std::collections::BTreeMap;

pub const DEFAULT_BRAVE_NEWS_ENDPOINT: &str = "https://api.search.brave.com/res/v1/news/search";
pub const DEFAULT_GDELT_ENDPOINT: &str = "https://api.gdeltproject.org/api/v2/doc/doc";
pub const DEFAULT_TIMEOUT_MS: u64 = 2_500;
pub const DEFAULT_MAX_RESULTS: usize = 6;
pub const DEFAULT_USER_AGENT: &str = "selene-news-provider-ladder/1.0";
pub const NEWS_ENGINE_ID: &str = "PH1.E";
const CACHE_SCHEMA_VERSION: &str = "1.0.0";
const DEFAULT_CACHE_POLICY_SNAPSHOT_ID: &str = "policy-snapshot-default";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum NewsProviderId {
    NewsProviderLadder,
    BraveNewsSearch,
    GdeltNewsAssist,
}

impl NewsProviderId {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::NewsProviderLadder => "news_provider_ladder",
            Self::BraveNewsSearch => "brave_news_search",
            Self::GdeltNewsAssist => "gdelt_news_assist",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NewsProviderErrorKind {
    BudgetExhausted,
    PolicyViolation,
    ProviderUnconfigured,
    DnsFailed,
    TlsFailed,
    ConnectFailed,
    TimeoutExceeded,
    HttpNon200,
    EmptyResults,
    ParseFailed,
    ProxyMisconfigured,
    HashCollisionDetected,
    TransportFailed,
}

impl NewsProviderErrorKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::BudgetExhausted => "budget_exhausted",
            Self::PolicyViolation => "policy_violation",
            Self::ProviderUnconfigured => "provider_unconfigured",
            Self::DnsFailed => "dns_failed",
            Self::TlsFailed => "tls_failed",
            Self::ConnectFailed => "connect_failed",
            Self::TimeoutExceeded => "timeout_exceeded",
            Self::HttpNon200 => "http_non_200",
            Self::EmptyResults => "empty_results",
            Self::ParseFailed => "parse_failed",
            Self::ProxyMisconfigured => "proxy_misconfigured",
            Self::HashCollisionDetected => "hash_collision_detected",
            Self::TransportFailed => "transport_failed",
        }
    }

    pub const fn reason_code(self) -> &'static str {
        match self {
            Self::BudgetExhausted => "budget_exhausted",
            Self::PolicyViolation => "policy_violation",
            Self::ProviderUnconfigured => "provider_unconfigured",
            Self::TimeoutExceeded => "timeout_exceeded",
            Self::EmptyResults => "empty_results",
            Self::ProxyMisconfigured => "proxy_misconfigured",
            Self::HashCollisionDetected => "hash_collision_detected",
            Self::DnsFailed
            | Self::TlsFailed
            | Self::ConnectFailed
            | Self::HttpNon200
            | Self::ParseFailed
            | Self::TransportFailed => "provider_upstream_failed",
        }
    }
}

#[derive(Debug, Clone)]
pub struct NewsProviderError {
    pub provider_id: NewsProviderId,
    pub kind: NewsProviderErrorKind,
    pub status_code: Option<u16>,
    pub message: String,
    pub latency_ms: u64,
}

impl NewsProviderError {
    pub const fn reason_code(&self) -> &'static str {
        self.kind.reason_code()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderNewsItem {
    pub title: String,
    pub url: String,
    pub snippet: String,
    pub canonical_url: String,
    pub published_raw: Option<String>,
    pub provider_id: NewsProviderId,
    pub provider_rank: usize,
    pub trust_tier: Option<String>,
    pub domain: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NormalizedNewsResult {
    pub title: String,
    pub url: String,
    pub snippet: String,
    pub canonical_url: String,
    pub published_at_ms: Option<i64>,
    pub published_at_utc: Option<String>,
    pub provider_id: NewsProviderId,
    pub provider_rank: usize,
    pub freshness_score: f64,
    pub trust_tier: Option<String>,
    pub domain: String,
}

#[derive(Debug, Clone)]
pub struct NewsRuntimeConfig {
    pub brave_news_endpoint: String,
    pub gdelt_endpoint: String,
    pub max_results: usize,
    pub timeout_ms: u64,
    pub user_agent: String,
    pub proxy_config: ProxyConfig,
    pub health_policy: HealthPolicy,
    pub brave_api_key_override: Option<String>,
}

impl NewsRuntimeConfig {
    pub fn from_env() -> Self {
        let env = SystemEnvProvider;
        let proxy_mode_raw =
            std::env::var("SELENE_NEWS_PROXY_MODE").unwrap_or_else(|_| "off".to_string());
        let proxy_mode = ProxyMode::parse(&proxy_mode_raw).unwrap_or(ProxyMode::Off);

        Self {
            brave_news_endpoint: std::env::var("BRAVE_SEARCH_NEWS_URL")
                .unwrap_or_else(|_| DEFAULT_BRAVE_NEWS_ENDPOINT.to_string()),
            gdelt_endpoint: std::env::var("GDELT_DOC_API_URL")
                .unwrap_or_else(|_| DEFAULT_GDELT_ENDPOINT.to_string()),
            max_results: DEFAULT_MAX_RESULTS,
            timeout_ms: DEFAULT_TIMEOUT_MS,
            user_agent: DEFAULT_USER_AGENT.to_string(),
            proxy_config: ProxyConfig::from_env(proxy_mode, &env),
            health_policy: HealthPolicy::default(),
            brave_api_key_override: None,
        }
    }
}

impl Default for NewsRuntimeConfig {
    fn default() -> Self {
        Self::from_env()
    }
}

#[derive(Debug, Clone)]
pub struct NewsAuditMetrics {
    pub lead_attempted: bool,
    pub fallback_used: bool,
    pub fallback_reason_code: Option<String>,
    pub recency_window_applied: i64,
    pub filtered_by_recency_count: usize,
    pub distinct_domain_count: usize,
    pub diversity_threshold_met: bool,
    pub contradiction_clusters_detected: usize,
    pub assist_used: bool,
}

#[derive(Debug, Clone)]
pub struct NewsProviderLadderResult {
    pub evidence_packet: Value,
    pub audit_metrics: NewsAuditMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct NewsProviderCallPayload {
    results: Vec<ProviderNewsItem>,
    latency_ms: u64,
}

pub fn execute_news_provider_ladder_from_tool_request(
    tool_request_packet: &Value,
    now_ms: i64,
    health_tracker: &mut ProviderHealthTracker,
    config: &NewsRuntimeConfig,
) -> Result<NewsProviderLadderResult, NewsProviderError> {
    let request = parse_tool_request(tool_request_packet)?;
    execute_news_provider_ladder(request, now_ms, health_tracker, config)
}

pub fn execute_news_ladder_from_tool_request(
    tool_request_packet: &Value,
    now_ms: i64,
    config: &NewsRuntimeConfig,
) -> Result<NewsProviderLadderResult, NewsProviderError> {
    let mut health_tracker = ProviderHealthTracker::default();
    execute_news_provider_ladder_from_tool_request(
        tool_request_packet,
        now_ms,
        &mut health_tracker,
        config,
    )
}

pub fn append_news_audit_fields(
    audit_packet: &mut Value,
    metrics: &NewsAuditMetrics,
) -> Result<(), String> {
    let obj = audit_packet
        .as_object_mut()
        .ok_or_else(|| "audit packet must be object".to_string())?;

    let transition_value = obj
        .entry("turn_state_transition".to_string())
        .or_insert_with(|| Value::Object(Map::new()));

    let transition_obj = if transition_value.is_object() {
        transition_value
            .as_object_mut()
            .ok_or_else(|| "turn_state_transition must be object".to_string())?
    } else if let Some(state) = transition_value.as_str() {
        *transition_value = json!({"state": state});
        transition_value
            .as_object_mut()
            .ok_or_else(|| "turn_state_transition conversion failed".to_string())?
    } else {
        return Err("turn_state_transition must be string or object".to_string());
    };

    transition_obj.insert(
        "news_audit".to_string(),
        json!({
            "lead_attempted": metrics.lead_attempted,
            "fallback_used": metrics.fallback_used,
            "fallback_reason_code": metrics.fallback_reason_code,
            "recency_window_applied": metrics.recency_window_applied,
            "filtered_by_recency_count": metrics.filtered_by_recency_count,
            "distinct_domain_count": metrics.distinct_domain_count,
            "diversity_threshold_met": metrics.diversity_threshold_met,
            "contradiction_clusters_detected": metrics.contradiction_clusters_detected,
            "assist_used": metrics.assist_used,
        }),
    );

    Ok(())
}

#[derive(Debug, Clone)]
struct ParsedToolRequest {
    trace_id: String,
    query: String,
    created_at_ms: i64,
    intended_consumers: Vec<String>,
    importance_tier: ImportanceTier,
    perf_importance_tier: PerfImportanceTier,
}

fn parse_tool_request(packet: &Value) -> Result<ParsedToolRequest, NewsProviderError> {
    let obj = packet.as_object().ok_or_else(|| NewsProviderError {
        provider_id: NewsProviderId::NewsProviderLadder,
        kind: NewsProviderErrorKind::ParseFailed,
        status_code: None,
        message: "tool request packet must be object".to_string(),
        latency_ms: 0,
    })?;

    let mode = obj.get("mode").and_then(Value::as_str).unwrap_or_default();
    if mode != "news" {
        return Err(NewsProviderError {
            provider_id: NewsProviderId::NewsProviderLadder,
            kind: NewsProviderErrorKind::ParseFailed,
            status_code: None,
            message: format!("tool request mode must be news, got {}", mode),
            latency_ms: 0,
        });
    }

    let query = obj
        .get("query")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| NewsProviderError {
            provider_id: NewsProviderId::NewsProviderLadder,
            kind: NewsProviderErrorKind::ParseFailed,
            status_code: None,
            message: "tool request query missing".to_string(),
            latency_ms: 0,
        })?
        .to_string();

    let trace_id = obj
        .get("trace_id")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| NewsProviderError {
            provider_id: NewsProviderId::NewsProviderLadder,
            kind: NewsProviderErrorKind::ParseFailed,
            status_code: None,
            message: "tool request trace_id missing".to_string(),
            latency_ms: 0,
        })?
        .to_string();

    let created_at_ms = obj
        .get("created_at_ms")
        .and_then(Value::as_i64)
        .ok_or_else(|| NewsProviderError {
            provider_id: NewsProviderId::NewsProviderLadder,
            kind: NewsProviderErrorKind::ParseFailed,
            status_code: None,
            message: "tool request created_at_ms missing".to_string(),
            latency_ms: 0,
        })?;

    let importance_tier = obj
        .get("importance_tier")
        .and_then(Value::as_str)
        .and_then(ImportanceTier::parse)
        .ok_or_else(|| NewsProviderError {
            provider_id: NewsProviderId::NewsProviderLadder,
            kind: NewsProviderErrorKind::ParseFailed,
            status_code: None,
            message: "tool request importance_tier missing or invalid".to_string(),
            latency_ms: 0,
        })?;

    let perf_importance_tier = obj
        .get("importance_tier")
        .and_then(Value::as_str)
        .map(PerfImportanceTier::parse_or_default)
        .unwrap_or_default();

    let intended_consumers = obj
        .get("intended_consumers")
        .and_then(Value::as_array)
        .map(|array| {
            array
                .iter()
                .filter_map(Value::as_str)
                .map(|entry| entry.trim().to_string())
                .filter(|entry| !entry.is_empty())
                .collect::<Vec<String>>()
        })
        .filter(|array| !array.is_empty())
        .unwrap_or_else(|| {
            vec![
                "PH1.D".to_string(),
                "PH1.WRITE".to_string(),
                "PH1.J".to_string(),
            ]
        });

    Ok(ParsedToolRequest {
        trace_id,
        query,
        created_at_ms,
        intended_consumers,
        importance_tier,
        perf_importance_tier,
    })
}

fn execute_news_provider_ladder(
    request: ParsedToolRequest,
    now_ms: i64,
    health_tracker: &mut ProviderHealthTracker,
    config: &NewsRuntimeConfig,
) -> Result<NewsProviderLadderResult, NewsProviderError> {
    if let Some(blocked) = stage2_news_provider_block(&request, config) {
        return Err(stage2_news_provider_error(
            NewsProviderId::NewsProviderLadder,
            blocked,
        ));
    }
    let window_days = recency_window_days(request.importance_tier);
    let tier_caps = caps_for_tier(request.perf_importance_tier);
    let effective_max_results = config.max_results.min(tier_caps.max_results_from_search);
    let effective_timeout_ms =
        clamp_provider_timeout(config.timeout_ms, request.perf_importance_tier);
    let mut provider_call_budget = ProviderCallBudget::for_tier(request.perf_importance_tier);
    let mut provider_runs = Vec::new();
    let mut l1_cache = L1Cache::default();
    let cache_enabled = news_cache_enabled();
    let cache_policy_snapshot_id = cache_policy_snapshot_id();
    let mut evidence_retrieved_at_ms = now_ms;

    let mut lead_attempted = false;
    let mut fallback_used = false;
    let mut fallback_reason_code: Option<String> = None;
    let mut assist_used = false;
    let mut filtered_total = 0usize;
    let fallback_enabled = provider_fallback_enabled_from_env()
        || (is_local_fake_endpoint(&config.brave_news_endpoint)
            && is_local_fake_endpoint(&config.gdelt_endpoint));

    let mut brave_filtered = Vec::new();
    let mut gdelt_filtered = Vec::new();

    let brave_state = health_tracker.snapshot(
        NewsProviderId::BraveNewsSearch.as_str(),
        now_ms,
        config.health_policy,
    );

    if health_tracker.should_skip_lead(
        NewsProviderId::BraveNewsSearch.as_str(),
        now_ms,
        config.health_policy,
    ) {
        if !fallback_enabled {
            return Err(NewsProviderError {
                provider_id: NewsProviderId::NewsProviderLadder,
                kind: NewsProviderErrorKind::PolicyViolation,
                status_code: None,
                message: "fallback_provider_disabled".to_string(),
                latency_ms: 0,
            });
        }
        fallback_used = true;
        assist_used = true;
        fallback_reason_code = Some("provider_upstream_failed".to_string());
        provider_runs.push(provider_run_skipped(
            NewsProviderId::BraveNewsSearch,
            "health_cooldown",
            brave_state,
            request.trace_id.as_str(),
            request.created_at_ms,
        ));

        provider_call_budget
            .record_fallback_call()
            .map_err(|reason| budget_exhausted_error(reason))?;
        let (gdelt, cache_hit) = run_news_provider_call_with_cache(
            &mut l1_cache,
            cache_enabled,
            &request,
            NewsProviderId::GdeltNewsAssist,
            cache_policy_snapshot_id.as_str(),
            now_ms,
            || {
                gdelt::execute_gdelt_news_search(
                    &config.gdelt_endpoint,
                    &request.query,
                    effective_max_results,
                    effective_timeout_ms,
                    &config.user_agent,
                    &config.proxy_config,
                )
                .map(|value| NewsProviderCallPayload {
                    results: value.results,
                    latency_ms: value.latency_ms,
                })
            },
        )?;
        if let Some(hit) = cache_hit {
            evidence_retrieved_at_ms = evidence_retrieved_at_ms.min(hit.retrieved_at_ms);
        }
        let (normalized, filtered_count) =
            normalize_and_filter_results(&gdelt.results, now_ms, window_days);
        filtered_total = filtered_total.saturating_add(filtered_count);
        gdelt_filtered = normalized;
        provider_runs.push(provider_run_success(
            NewsProviderId::GdeltNewsAssist,
            gdelt.latency_ms,
            gdelt_filtered.len(),
            Some("health_cooldown"),
            health_tracker.snapshot(
                NewsProviderId::GdeltNewsAssist.as_str(),
                now_ms,
                config.health_policy,
            ),
        ));
    } else {
        lead_attempted = true;
        provider_call_budget
            .record_lead_call()
            .map_err(|reason| budget_exhausted_error(reason))?;
        let brave_key = resolve_brave_api_key(config)?;
        match run_news_provider_call_with_cache(
            &mut l1_cache,
            cache_enabled,
            &request,
            NewsProviderId::BraveNewsSearch,
            cache_policy_snapshot_id.as_str(),
            now_ms,
            || {
                brave_news::execute_brave_news_search(
                    &config.brave_news_endpoint,
                    &brave_key,
                    &request.query,
                    effective_max_results,
                    effective_timeout_ms,
                    &config.user_agent,
                    &config.proxy_config,
                )
                .map(|value| NewsProviderCallPayload {
                    results: value.results,
                    latency_ms: value.latency_ms,
                })
            },
        ) {
            Ok(brave) => {
                let (brave, cache_hit) = brave;
                if let Some(hit) = cache_hit {
                    evidence_retrieved_at_ms = evidence_retrieved_at_ms.min(hit.retrieved_at_ms);
                }
                health_tracker.record_success(NewsProviderId::BraveNewsSearch.as_str());
                let (normalized, filtered_count) =
                    normalize_and_filter_results(&brave.results, now_ms, window_days);
                filtered_total = filtered_total.saturating_add(filtered_count);
                brave_filtered = normalized;
                provider_runs.push(provider_run_success(
                    NewsProviderId::BraveNewsSearch,
                    brave.latency_ms,
                    brave_filtered.len(),
                    None,
                    health_tracker.snapshot(
                        NewsProviderId::BraveNewsSearch.as_str(),
                        now_ms,
                        config.health_policy,
                    ),
                ));
            }
            Err(err) => {
                health_tracker.record_failure(
                    NewsProviderId::BraveNewsSearch.as_str(),
                    now_ms,
                    config.health_policy,
                );

                let fallback_trigger = fallback_trigger_label(err.kind);
                let can_fallback = fallback_enabled && fallback_trigger.is_some();
                if can_fallback {
                    fallback_used = true;
                    assist_used = true;
                    fallback_reason_code = Some(err.reason_code().to_string());
                }

                provider_runs.push(provider_run_error(
                    &err,
                    0,
                    fallback_trigger,
                    health_tracker.snapshot(
                        NewsProviderId::BraveNewsSearch.as_str(),
                        now_ms,
                        config.health_policy,
                    ),
                    request.trace_id.as_str(),
                    request.created_at_ms,
                ));

                if can_fallback {
                    provider_call_budget
                        .record_fallback_call()
                        .map_err(|reason| budget_exhausted_error(reason))?;
                    match run_news_provider_call_with_cache(
                        &mut l1_cache,
                        cache_enabled,
                        &request,
                        NewsProviderId::GdeltNewsAssist,
                        cache_policy_snapshot_id.as_str(),
                        now_ms,
                        || {
                            gdelt::execute_gdelt_news_search(
                                &config.gdelt_endpoint,
                                &request.query,
                                effective_max_results,
                                effective_timeout_ms,
                                &config.user_agent,
                                &config.proxy_config,
                            )
                            .map(|value| NewsProviderCallPayload {
                                results: value.results,
                                latency_ms: value.latency_ms,
                            })
                        },
                    ) {
                        Ok(gdelt) => {
                            let (gdelt, cache_hit) = gdelt;
                            if let Some(hit) = cache_hit {
                                evidence_retrieved_at_ms =
                                    evidence_retrieved_at_ms.min(hit.retrieved_at_ms);
                            }
                            health_tracker.record_success(NewsProviderId::GdeltNewsAssist.as_str());
                            let (normalized, filtered_count) =
                                normalize_and_filter_results(&gdelt.results, now_ms, window_days);
                            filtered_total = filtered_total.saturating_add(filtered_count);
                            gdelt_filtered = normalized;
                            provider_runs.push(provider_run_success(
                                NewsProviderId::GdeltNewsAssist,
                                gdelt.latency_ms,
                                gdelt_filtered.len(),
                                fallback_trigger,
                                health_tracker.snapshot(
                                    NewsProviderId::GdeltNewsAssist.as_str(),
                                    now_ms,
                                    config.health_policy,
                                ),
                            ));
                        }
                        Err(gdelt_err) => {
                            health_tracker.record_failure(
                                NewsProviderId::GdeltNewsAssist.as_str(),
                                now_ms,
                                config.health_policy,
                            );
                            provider_runs.push(provider_run_error(
                                &gdelt_err,
                                0,
                                fallback_trigger,
                                health_tracker.snapshot(
                                    NewsProviderId::GdeltNewsAssist.as_str(),
                                    now_ms,
                                    config.health_policy,
                                ),
                                request.trace_id.as_str(),
                                request.created_at_ms,
                            ));
                            return Err(gdelt_err);
                        }
                    }
                } else {
                    return Err(err);
                }
            }
        }

        let brave_domain_count = distinct_domain_count(&brave_filtered);
        let needs_assist_for_insufficiency = brave_filtered.is_empty()
            || (request.importance_tier == ImportanceTier::High
                && !diversity_threshold_met(request.importance_tier, brave_domain_count));

        if !assist_used && needs_assist_for_insufficiency && fallback_enabled {
            fallback_used = true;
            assist_used = true;
            fallback_reason_code = Some(if brave_filtered.is_empty() {
                "empty_results".to_string()
            } else {
                "insufficient_evidence".to_string()
            });

            provider_call_budget
                .record_fallback_call()
                .map_err(|reason| budget_exhausted_error(reason))?;
            match run_news_provider_call_with_cache(
                &mut l1_cache,
                cache_enabled,
                &request,
                NewsProviderId::GdeltNewsAssist,
                cache_policy_snapshot_id.as_str(),
                now_ms,
                || {
                    gdelt::execute_gdelt_news_search(
                        &config.gdelt_endpoint,
                        &request.query,
                        effective_max_results,
                        effective_timeout_ms,
                        &config.user_agent,
                        &config.proxy_config,
                    )
                    .map(|value| NewsProviderCallPayload {
                        results: value.results,
                        latency_ms: value.latency_ms,
                    })
                },
            ) {
                Ok(gdelt) => {
                    let (gdelt, cache_hit) = gdelt;
                    if let Some(hit) = cache_hit {
                        evidence_retrieved_at_ms =
                            evidence_retrieved_at_ms.min(hit.retrieved_at_ms);
                    }
                    health_tracker.record_success(NewsProviderId::GdeltNewsAssist.as_str());
                    let (normalized, filtered_count) =
                        normalize_and_filter_results(&gdelt.results, now_ms, window_days);
                    filtered_total = filtered_total.saturating_add(filtered_count);
                    gdelt_filtered = normalized;
                    provider_runs.push(provider_run_success(
                        NewsProviderId::GdeltNewsAssist,
                        gdelt.latency_ms,
                        gdelt_filtered.len(),
                        Some("insufficient_recall"),
                        health_tracker.snapshot(
                            NewsProviderId::GdeltNewsAssist.as_str(),
                            now_ms,
                            config.health_policy,
                        ),
                    ));
                }
                Err(err) => {
                    health_tracker.record_failure(
                        NewsProviderId::GdeltNewsAssist.as_str(),
                        now_ms,
                        config.health_policy,
                    );
                    provider_runs.push(provider_run_error(
                        &err,
                        0,
                        Some("insufficient_recall"),
                        health_tracker.snapshot(
                            NewsProviderId::GdeltNewsAssist.as_str(),
                            now_ms,
                            config.health_policy,
                        ),
                        request.trace_id.as_str(),
                        request.created_at_ms,
                    ));
                    if brave_filtered.is_empty() {
                        return Err(err);
                    }
                }
            }
        }
    }

    let merge = merge_news_results(&brave_filtered, &gdelt_filtered);
    if merge.merged_results.is_empty() {
        let error = NewsProviderError {
            provider_id: NewsProviderId::NewsProviderLadder,
            kind: NewsProviderErrorKind::EmptyResults,
            status_code: None,
            message: "no news results after recency filtering and merge".to_string(),
            latency_ms: 0,
        };
        let _ = try_build_debug_packet(DebugPacketContext {
            trace_id: request.trace_id.as_str(),
            status: DebugStatus::Failed,
            provider: "Planning",
            error_kind: error.kind.as_str(),
            reason_code: error.reason_code(),
            proxy_mode: None,
            source_url: None,
            created_at_ms: request.created_at_ms,
            turn_state_transitions: &default_failed_transitions(request.created_at_ms),
            debug_hint: Some(error.message.as_str()),
            fallback_used: Some(fallback_used),
            health_status_before_fallback: Some(health_to_debug_status(brave_state)),
        });
        return Err(error);
    }

    let domain_count = distinct_domain_count(&merge.merged_results);
    let diversity_met = diversity_threshold_met(request.importance_tier, domain_count);

    let clusters = build_conflict_clusters(&merge.merged_results);
    let contradiction_lookup = cluster_lookup_by_canonical_url(&merge.merged_results, &clusters);

    let sources: Vec<Value> = merge
        .merged_results
        .iter()
        .enumerate()
        .map(|(idx, result)| {
            let contradiction_group_id = contradiction_lookup.get(&result.canonical_url).cloned();
            let corroborated_by_assist = merge
                .corroborated_canonical_urls
                .contains(&result.canonical_url);
            json!({
                "title": result.title.clone(),
                "url": result.url.clone(),
                "snippet": result.snippet.clone(),
                "published_at": result.published_at_utc.clone(),
                "published_at_ms": result.published_at_ms,
                "media_type": "news",
                "provider_id": result.provider_id.as_str(),
                "rank": idx + 1,
                "canonical_url": result.canonical_url.clone(),
                "freshness_score": result.freshness_score,
                "trust_tier": result.trust_tier.clone(),
                "corroborated_by_assist": corroborated_by_assist,
                "contradiction_group_id": contradiction_group_id,
            })
        })
        .collect();

    let content_chunks = build_news_content_chunks(&merge.merged_results, &contradiction_lookup)?;

    let trust_metadata_clusters: Vec<Value> = clusters
        .iter()
        .map(|cluster| {
            json!({
                "group_id": cluster.group_id.clone(),
                "topic_key": cluster.topic_key.clone(),
                "source_refs": cluster.source_refs.clone(),
                "conflicting_claims": cluster.conflicting_claims.clone(),
            })
        })
        .collect();

    let evidence_packet = json!({
        "schema_version": "1.0.0",
        "produced_by": NEWS_ENGINE_ID,
        "intended_consumers": request.intended_consumers,
        "created_at_ms": request.created_at_ms,
        "trace_id": request.trace_id,
        "query": request.query,
        "retrieved_at_ms": evidence_retrieved_at_ms,
        "provider_runs": provider_runs,
        "sources": sources,
        "content_chunks": content_chunks,
        "trust_metadata": {
            "news_provider_ladder": {
                "lead_provider": NewsProviderId::BraveNewsSearch.as_str(),
                "assist_provider": NewsProviderId::GdeltNewsAssist.as_str(),
                "lead_attempted": lead_attempted,
                "fallback_used": fallback_used,
                "fallback_reason_code": fallback_reason_code,
                "assist_used": assist_used,
                "recency_window_applied": window_days,
                "filtered_by_recency_count": filtered_total,
                "distinct_domain_count": domain_count,
                "diversity_threshold_met": diversity_met,
                "dedup_count": merge.dedup_count,
                "corroborated_count": merge.corroborated_canonical_urls.len(),
                "contradiction_clusters": trust_metadata_clusters,
                "importance_tier": request.perf_importance_tier.as_str(),
                "max_total_provider_calls_per_turn": tier_caps.max_total_provider_calls_per_turn,
                "max_fallback_invocations_per_turn": tier_caps.max_fallback_invocations_per_turn,
                "max_retries_per_provider": tier_caps.max_retries_per_provider,
                "total_provider_calls": provider_call_budget.total_provider_calls(),
                "fallback_invocations": provider_call_budget.fallback_invocations(),
                "timeout_per_provider_ms": effective_timeout_ms,
            }
        }
    });

    let audit_metrics = NewsAuditMetrics {
        lead_attempted,
        fallback_used,
        fallback_reason_code,
        recency_window_applied: window_days,
        filtered_by_recency_count: filtered_total,
        distinct_domain_count: domain_count,
        diversity_threshold_met: diversity_met,
        contradiction_clusters_detected: clusters.len(),
        assist_used,
    };

    Ok(NewsProviderLadderResult {
        evidence_packet,
        audit_metrics,
    })
}

fn stage2_news_provider_block(
    request: &ParsedToolRequest,
    config: &NewsRuntimeConfig,
) -> Option<ProviderGateDecision> {
    let fake_endpoint = is_local_fake_endpoint(&config.brave_news_endpoint)
        && is_local_fake_endpoint(&config.gdelt_endpoint);
    let decision = if fake_endpoint {
        fake_provider_decision(
            ProviderControlRoute::NewsSearch,
            ProviderControlProvider::BraveNewsSearch,
            &request.query,
            1,
        )
    } else {
        disabled_provider_decision(
            ProviderControlRoute::NewsSearch,
            ProviderControlProvider::BraveNewsSearch,
            &request.query,
        )
    };
    (!decision.allowed).then_some(decision)
}

fn stage2_news_provider_error(
    provider_id: NewsProviderId,
    decision: ProviderGateDecision,
) -> NewsProviderError {
    NewsProviderError {
        provider_id,
        kind: NewsProviderErrorKind::PolicyViolation,
        status_code: None,
        message: decision.disabled_trace_line(),
        latency_ms: 0,
    }
}

fn normalize_and_filter_results(
    items: &[ProviderNewsItem],
    now_ms: i64,
    window_days: i64,
) -> (Vec<NormalizedNewsResult>, usize) {
    let mut out = Vec::new();
    let mut filtered = 0usize;

    for item in items {
        let normalized_published = item
            .published_raw
            .as_deref()
            .and_then(normalize_published_at);

        let published_ms = normalized_published.as_ref().map(|p| p.epoch_ms);
        let published_utc = normalized_published.map(|p| p.utc_rfc3339);

        if !within_recency_window(published_ms, now_ms, window_days) {
            filtered = filtered.saturating_add(1);
            continue;
        }

        out.push(NormalizedNewsResult {
            title: item.title.clone(),
            url: item.url.clone(),
            snippet: item.snippet.clone(),
            canonical_url: item.canonical_url.clone(),
            published_at_ms: published_ms,
            published_at_utc: published_utc,
            provider_id: item.provider_id,
            provider_rank: item.provider_rank,
            freshness_score: freshness_score(published_ms, now_ms, window_days),
            trust_tier: item.trust_tier.clone(),
            domain: item.domain.clone(),
        });
    }

    (out, filtered)
}

fn build_news_content_chunks(
    results: &[NormalizedNewsResult],
    contradiction_lookup: &BTreeMap<String, String>,
) -> Result<Vec<Value>, NewsProviderError> {
    let mut chunks = Vec::new();
    let mut collision_guard: BTreeMap<String, String> = BTreeMap::new();

    for (idx, result) in results.iter().enumerate() {
        let normalized = normalize_document_for_chunking(&result.snippet);
        if normalized.is_empty() {
            continue;
        }

        let text_chunk = TextChunk {
            chunk_index: idx,
            normalized_text: normalized.clone(),
            norm_version: NORM_VERSION,
            chunk_version: CHUNK_VERSION,
        };

        let chunk_id = derive_chunk_id(&result.canonical_url, &text_chunk, &Sha256ChunkHasher);
        if let Some(previous) = collision_guard.get(chunk_id.as_str()) {
            if previous != &normalized {
                return Err(NewsProviderError {
                    provider_id: NewsProviderId::NewsProviderLadder,
                    kind: NewsProviderErrorKind::HashCollisionDetected,
                    status_code: None,
                    message: format!("news chunk collision for chunk_id {}", chunk_id),
                    latency_ms: 0,
                });
            }
        } else {
            collision_guard.insert(chunk_id.clone(), normalized.clone());
        }

        let chunk_id_for_citation = chunk_id.clone();
        let source_url = result.url.clone();
        let canonical_url = result.canonical_url.clone();
        let contradiction_group_id = contradiction_lookup.get(&result.canonical_url).cloned();
        chunks.push(json!({
            "chunk_id": chunk_id,
            "hash_version": HASH_VERSION,
            "norm_version": NORM_VERSION,
            "chunk_version": CHUNK_VERSION,
            "source_url": source_url.clone(),
            "canonical_url": canonical_url,
            "chunk_index": idx,
            "text_excerpt": bounded_excerpt(&normalized, 320),
            "text_len_chars": normalized.chars().count(),
            "published_at": result.published_at_utc.clone(),
            "contradiction_group_id": contradiction_group_id,
            "citation": {
                "chunk_id": chunk_id_for_citation,
                "source_url": source_url,
            }
        }));
    }

    Ok(chunks)
}

fn provider_run_success(
    provider_id: NewsProviderId,
    latency_ms: u64,
    results_count: usize,
    fallback_trigger: Option<&str>,
    health_state: ProviderHealthState,
) -> Value {
    json!({
        "provider_id": provider_id.as_str(),
        "endpoint": "news",
        "latency_ms": latency_ms,
        "results_count": results_count,
        "fallback_trigger": fallback_trigger,
        "health_state": health_state.as_str(),
        "error": Value::Null,
    })
}

fn provider_run_error(
    error: &NewsProviderError,
    results_count: usize,
    fallback_trigger: Option<&str>,
    health_state: ProviderHealthState,
    trace_id: &str,
    created_at_ms: i64,
) -> Value {
    let degraded = fallback_trigger.is_some();
    let status = if degraded {
        DebugStatus::Degraded
    } else {
        DebugStatus::Failed
    };
    let transitions = if degraded {
        default_degraded_transitions(created_at_ms)
    } else {
        default_failed_transitions(created_at_ms)
    };
    let debug_packet = try_build_debug_packet(DebugPacketContext {
        trace_id,
        status,
        provider: provider_name_for_debug(error.provider_id),
        error_kind: error.kind.as_str(),
        reason_code: error.reason_code(),
        proxy_mode: None,
        source_url: None,
        created_at_ms,
        turn_state_transitions: &transitions,
        debug_hint: Some(error.message.as_str()),
        fallback_used: Some(degraded),
        health_status_before_fallback: Some(health_to_debug_status(health_state)),
    })
    .ok()
    .and_then(|packet| serde_json::to_value(packet).ok())
    .unwrap_or(Value::Null);

    json!({
        "provider_id": error.provider_id.as_str(),
        "endpoint": "news",
        "latency_ms": error.latency_ms,
        "results_count": results_count,
        "fallback_trigger": fallback_trigger,
        "health_state": health_state.as_str(),
        "error": {
            "error_kind": error.kind.as_str(),
            "reason_code": error.reason_code(),
            "status_code": error.status_code,
            "message": error.message,
            "debug_packet": debug_packet,
        }
    })
}

fn provider_run_skipped(
    provider_id: NewsProviderId,
    fallback_trigger: &str,
    health_state: ProviderHealthState,
    trace_id: &str,
    created_at_ms: i64,
) -> Value {
    let transitions = default_degraded_transitions(created_at_ms);
    let debug_packet = try_build_debug_packet(DebugPacketContext {
        trace_id,
        status: DebugStatus::Degraded,
        provider: provider_name_for_debug(provider_id),
        error_kind: "health_cooldown",
        reason_code: "provider_upstream_failed",
        proxy_mode: None,
        source_url: None,
        created_at_ms,
        turn_state_transitions: &transitions,
        debug_hint: Some("provider skipped due to cooldown"),
        fallback_used: Some(true),
        health_status_before_fallback: Some(health_to_debug_status(health_state)),
    })
    .ok()
    .and_then(|packet| serde_json::to_value(packet).ok())
    .unwrap_or(Value::Null);

    json!({
        "provider_id": provider_id.as_str(),
        "endpoint": "news",
        "latency_ms": 0,
        "results_count": 0,
        "fallback_trigger": fallback_trigger,
        "health_state": health_state.as_str(),
        "error": Value::Null,
        "skipped_due_to_cooldown": true,
        "debug_packet": debug_packet,
    })
}

fn provider_name_for_debug(provider_id: NewsProviderId) -> &'static str {
    match provider_id {
        NewsProviderId::NewsProviderLadder => "Planning",
        NewsProviderId::BraveNewsSearch => "BraveWebSearch",
        NewsProviderId::GdeltNewsAssist => "GDELT",
    }
}

fn health_to_debug_status(state: ProviderHealthState) -> HealthStatusBeforeFallback {
    match state {
        ProviderHealthState::Healthy => HealthStatusBeforeFallback::Healthy,
        ProviderHealthState::Degraded => HealthStatusBeforeFallback::Degraded,
        ProviderHealthState::Cooldown => HealthStatusBeforeFallback::Cooldown,
    }
}

fn fallback_trigger_label(kind: NewsProviderErrorKind) -> Option<&'static str> {
    if should_trigger_fallback(kind) {
        Some(kind.as_str())
    } else {
        None
    }
}

fn should_trigger_fallback(kind: NewsProviderErrorKind) -> bool {
    matches!(
        kind,
        NewsProviderErrorKind::DnsFailed
            | NewsProviderErrorKind::TlsFailed
            | NewsProviderErrorKind::ConnectFailed
            | NewsProviderErrorKind::TimeoutExceeded
            | NewsProviderErrorKind::HttpNon200
            | NewsProviderErrorKind::EmptyResults
            | NewsProviderErrorKind::ParseFailed
    )
}

fn resolve_brave_api_key(config: &NewsRuntimeConfig) -> Result<String, NewsProviderError> {
    config
        .brave_api_key_override
        .clone()
        .or_else(|| resolve_secret_from_vault(ProviderSecretId::BraveSearchApiKey))
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| NewsProviderError {
            provider_id: NewsProviderId::BraveNewsSearch,
            kind: NewsProviderErrorKind::ProviderUnconfigured,
            status_code: None,
            message: "missing brave_search_api_key in device vault".to_string(),
            latency_ms: 0,
        })
}

fn resolve_secret_from_vault(secret_id: ProviderSecretId) -> Option<String> {
    match selene_engines::device_vault::resolve_secret(secret_id.as_str()) {
        Ok(Some(secret)) => {
            let trimmed = secret.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_string())
            }
        }
        _ => None,
    }
}

fn run_news_provider_call_with_cache<F>(
    l1_cache: &mut L1Cache,
    cache_enabled: bool,
    request: &ParsedToolRequest,
    provider_id: NewsProviderId,
    policy_snapshot_id: &str,
    now_ms: i64,
    call: F,
) -> Result<
    (
        NewsProviderCallPayload,
        Option<CacheLookupHit<NewsProviderCallPayload>>,
    ),
    NewsProviderError,
>
where
    F: FnOnce() -> Result<NewsProviderCallPayload, NewsProviderError>,
{
    if cache_enabled {
        let key = CacheKey::new(
            CacheMode::News,
            request.query.as_str(),
            None,
            Some(provider_id.as_str()),
            request.perf_importance_tier,
            Some(policy_snapshot_id),
        );
        match lookup_typed::<NewsProviderCallPayload>(
            l1_cache,
            &key,
            now_ms,
            CACHE_SCHEMA_VERSION,
            policy_snapshot_id,
        ) {
            Ok(Some(hit)) => return Ok((hit.value.clone(), Some(hit))),
            Ok(None) => {}
            Err(err) => {
                return Err(NewsProviderError {
                    provider_id,
                    kind: NewsProviderErrorKind::PolicyViolation,
                    status_code: None,
                    message: format!("cache lookup rejected: {}", err),
                    latency_ms: 0,
                })
            }
        }

        let fresh = call()?;
        let ttl_ms = ttl_ms_for(CacheMode::News, request.perf_importance_tier);
        store_typed(
            l1_cache,
            &key,
            &fresh,
            CACHE_SCHEMA_VERSION,
            now_ms,
            ttl_ms,
            policy_snapshot_id,
            now_ms,
        )
        .map_err(|err| NewsProviderError {
            provider_id,
            kind: NewsProviderErrorKind::PolicyViolation,
            status_code: None,
            message: format!("cache store rejected: {}", err),
            latency_ms: 0,
        })?;

        return Ok((fresh, None));
    }

    call().map(|fresh| (fresh, None))
}

fn news_cache_enabled() -> bool {
    match std::env::var("SELENE_NEWS_CACHE_ENABLED") {
        Ok(value) => matches!(
            value.trim().to_ascii_lowercase().as_str(),
            "1" | "true" | "yes" | "on"
        ),
        Err(_) => false,
    }
}

fn cache_policy_snapshot_id() -> String {
    std::env::var("SELENE_NEWS_CACHE_POLICY_SNAPSHOT_ID")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| DEFAULT_CACHE_POLICY_SNAPSHOT_ID.to_string())
}

fn budget_exhausted_error(message: &str) -> NewsProviderError {
    NewsProviderError {
        provider_id: NewsProviderId::NewsProviderLadder,
        kind: NewsProviderErrorKind::BudgetExhausted,
        status_code: None,
        message: format!("provider call budget exhausted: {}", message),
        latency_ms: 0,
    }
}

pub fn normalize_text_value(input: &str) -> String {
    input
        .replace('\r', " ")
        .replace('\n', " ")
        .split_whitespace()
        .collect::<Vec<&str>>()
        .join(" ")
        .trim()
        .to_string()
}

pub fn extract_domain(raw_url: &str) -> Option<String> {
    url::Url::parse(raw_url)
        .ok()
        .and_then(|url| url.host_str().map(|host| host.to_ascii_lowercase()))
}

#[cfg(test)]
pub mod news_tests;

#[cfg(test)]
pub mod news_parity_tests;
