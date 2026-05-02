#![forbid(unsafe_code)]

pub mod brave_adapter;
pub mod fallback_policy;
pub mod health_state;
pub mod openai_fallback;
pub mod provider_merge;

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
use crate::web_search_plan::perf_cost::budgets::ProviderCallBudget;
use crate::web_search_plan::perf_cost::tiers::{caps_for_tier, ImportanceTier};
use crate::web_search_plan::perf_cost::timeouts::clamp_provider_timeout;
use crate::web_search_plan::proxy::proxy_config::{ProxyConfig, SystemEnvProvider};
use crate::web_search_plan::proxy::ProxyMode;
use crate::web_search_plan::web_provider::fallback_policy::{
    fallback_trigger_label, should_trigger_fallback,
};
use crate::web_search_plan::web_provider::health_state::{
    HealthPolicy, ProviderHealthState, ProviderHealthTracker,
};
use crate::web_search_plan::web_provider::provider_merge::merge_results;
use selene_engines::ph1providerctl::{
    disabled_provider_decision, fake_provider_decision, is_local_fake_endpoint,
    provider_fallback_enabled_from_env, ProviderControlProvider, ProviderControlRoute,
    ProviderGateDecision,
};
use selene_kernel_contracts::provider_secrets::ProviderSecretId;
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};
use std::collections::BTreeMap;

pub const DEFAULT_BRAVE_WEB_ENDPOINT: &str = "https://api.search.brave.com/res/v1/web/search";
pub const DEFAULT_OPENAI_RESPONSES_ENDPOINT: &str = "https://api.openai.com/v1/responses";
pub const DEFAULT_OPENAI_WEB_MODEL: &str = "gpt-4o-mini";
pub const DEFAULT_MAX_RESULTS: usize = 5;
pub const DEFAULT_TIMEOUT_MS: u64 = 2_500;
pub const DEFAULT_USER_AGENT: &str = "selene-web-provider-ladder/1.0";
pub const WEB_PROVIDER_ENGINE_ID: &str = "PH1.E";
const CACHE_SCHEMA_VERSION: &str = "1.0.0";
const DEFAULT_CACHE_POLICY_SNAPSHOT_ID: &str = "policy-snapshot-default";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ProviderId {
    WebProviderLadder,
    BraveWebSearch,
    OpenAiWebSearch,
}

impl ProviderId {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::WebProviderLadder => "web_provider_ladder",
            Self::BraveWebSearch => "brave_web_search",
            Self::OpenAiWebSearch => "openai_web_search",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderErrorKind {
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

impl ProviderErrorKind {
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
pub struct ProviderError {
    pub provider_id: ProviderId,
    pub kind: ProviderErrorKind,
    pub status_code: Option<u16>,
    pub message: String,
    pub latency_ms: u64,
}

impl ProviderError {
    pub const fn reason_code(&self) -> &'static str {
        self.kind.reason_code()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NormalizedSearchResult {
    pub title: String,
    pub url: String,
    pub snippet: String,
    pub canonical_url: String,
    pub citation_url: String,
    pub provider_id: ProviderId,
    pub provider_rank: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderCallSuccess {
    pub results: Vec<NormalizedSearchResult>,
    pub latency_ms: u64,
}

#[derive(Debug, Clone)]
pub struct WebProviderRuntimeConfig {
    pub brave_endpoint: String,
    pub openai_endpoint: String,
    pub openai_model: String,
    pub max_results: usize,
    pub timeout_ms: u64,
    pub user_agent: String,
    pub proxy_config: ProxyConfig,
    pub health_policy: HealthPolicy,
    pub brave_api_key_override: Option<String>,
    pub openai_api_key_override: Option<String>,
}

impl WebProviderRuntimeConfig {
    pub fn from_env() -> Self {
        let env = SystemEnvProvider;
        let proxy_mode_raw =
            std::env::var("SELENE_WEB_PROXY_MODE").unwrap_or_else(|_| "off".to_string());
        let proxy_mode = ProxyMode::parse(&proxy_mode_raw).unwrap_or(ProxyMode::Off);

        Self {
            brave_endpoint: std::env::var("BRAVE_SEARCH_WEB_URL")
                .unwrap_or_else(|_| DEFAULT_BRAVE_WEB_ENDPOINT.to_string()),
            openai_endpoint: std::env::var("OPENAI_RESPONSES_URL")
                .unwrap_or_else(|_| DEFAULT_OPENAI_RESPONSES_ENDPOINT.to_string()),
            openai_model: std::env::var("OPENAI_WEB_FALLBACK_MODEL")
                .unwrap_or_else(|_| DEFAULT_OPENAI_WEB_MODEL.to_string()),
            max_results: DEFAULT_MAX_RESULTS,
            timeout_ms: DEFAULT_TIMEOUT_MS,
            user_agent: DEFAULT_USER_AGENT.to_string(),
            proxy_config: ProxyConfig::from_env(proxy_mode, &env),
            health_policy: HealthPolicy::default(),
            brave_api_key_override: None,
            openai_api_key_override: None,
        }
    }
}

impl Default for WebProviderRuntimeConfig {
    fn default() -> Self {
        Self::from_env()
    }
}

#[derive(Debug, Clone)]
pub struct WebProviderAuditMetrics {
    pub lead_attempted: bool,
    pub fallback_used: bool,
    pub fallback_reason_code: Option<String>,
    pub provider_health_snapshot: Value,
    pub results_count_lead: usize,
    pub results_count_fallback: usize,
    pub dedup_count: usize,
}

#[derive(Debug, Clone)]
pub struct WebProviderLadderResult {
    pub evidence_packet: Value,
    pub audit_metrics: WebProviderAuditMetrics,
}

pub fn execute_web_provider_ladder_from_tool_request(
    tool_request_packet: &Value,
    now_ms: i64,
    health_tracker: &mut ProviderHealthTracker,
    config: &WebProviderRuntimeConfig,
) -> Result<WebProviderLadderResult, ProviderError> {
    let parsed = parse_tool_request_packet(tool_request_packet)?;
    execute_web_provider_ladder(parsed, now_ms, health_tracker, config)
}

pub fn append_web_provider_audit_fields(
    audit_packet: &mut Value,
    metrics: &WebProviderAuditMetrics,
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
        "web_provider_audit".to_string(),
        json!({
            "lead_attempted": metrics.lead_attempted,
            "fallback_used": metrics.fallback_used,
            "fallback_reason_code": metrics.fallback_reason_code,
            "provider_health_snapshot": metrics.provider_health_snapshot,
            "results_count_lead": metrics.results_count_lead,
            "results_count_fallback": metrics.results_count_fallback,
            "dedup_count": metrics.dedup_count,
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
}

fn parse_tool_request_packet(packet: &Value) -> Result<ParsedToolRequest, ProviderError> {
    let obj = packet.as_object().ok_or_else(|| ProviderError {
        provider_id: ProviderId::WebProviderLadder,
        kind: ProviderErrorKind::ParseFailed,
        status_code: None,
        message: "tool request packet must be object".to_string(),
        latency_ms: 0,
    })?;

    let mode = obj.get("mode").and_then(Value::as_str).unwrap_or_default();
    if mode != "web" {
        return Err(ProviderError {
            provider_id: ProviderId::WebProviderLadder,
            kind: ProviderErrorKind::ParseFailed,
            status_code: None,
            message: format!("tool request mode must be web, got {}", mode),
            latency_ms: 0,
        });
    }

    let query = obj
        .get("query")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|q| !q.is_empty())
        .ok_or_else(|| ProviderError {
            provider_id: ProviderId::WebProviderLadder,
            kind: ProviderErrorKind::ParseFailed,
            status_code: None,
            message: "tool request query is missing".to_string(),
            latency_ms: 0,
        })?
        .to_string();

    let trace_id = obj
        .get("trace_id")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .ok_or_else(|| ProviderError {
            provider_id: ProviderId::WebProviderLadder,
            kind: ProviderErrorKind::ParseFailed,
            status_code: None,
            message: "tool request trace_id is missing".to_string(),
            latency_ms: 0,
        })?
        .to_string();

    let created_at_ms = obj
        .get("created_at_ms")
        .and_then(Value::as_i64)
        .ok_or_else(|| ProviderError {
            provider_id: ProviderId::WebProviderLadder,
            kind: ProviderErrorKind::ParseFailed,
            status_code: None,
            message: "tool request created_at_ms is missing".to_string(),
            latency_ms: 0,
        })?;

    let importance_tier = obj
        .get("importance_tier")
        .and_then(Value::as_str)
        .map(ImportanceTier::parse_or_default)
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
    })
}

fn execute_web_provider_ladder(
    request: ParsedToolRequest,
    now_ms: i64,
    health_tracker: &mut ProviderHealthTracker,
    config: &WebProviderRuntimeConfig,
) -> Result<WebProviderLadderResult, ProviderError> {
    if let Some(blocked) = stage2_web_provider_block(&request, config) {
        return Err(stage2_provider_error(
            ProviderId::WebProviderLadder,
            blocked,
        ));
    }
    let brave_state = health_tracker.snapshot(
        ProviderId::BraveWebSearch.as_str(),
        now_ms,
        config.health_policy,
    );
    let mut lead_attempted = false;
    let mut fallback_used = false;
    let mut fallback_reason_code = None;
    let fallback_enabled = provider_fallback_enabled_from_env()
        || (is_local_fake_endpoint(&config.brave_endpoint)
            && is_local_fake_endpoint(&config.openai_endpoint));

    let mut provider_runs = Vec::new();
    let mut brave_results = Vec::new();
    let mut openai_results = Vec::new();
    let mut results_count_lead = 0usize;
    let mut results_count_fallback = 0usize;
    let tier_caps = caps_for_tier(request.importance_tier);
    let effective_max_results = config.max_results.min(tier_caps.max_results_from_search);
    let effective_timeout_ms = clamp_provider_timeout(config.timeout_ms, request.importance_tier);
    let mut provider_call_budget = ProviderCallBudget::for_tier(request.importance_tier);
    let mut l1_cache = L1Cache::default();
    let cache_enabled = web_cache_enabled();
    let cache_policy_snapshot_id = cache_policy_snapshot_id();
    let mut evidence_retrieved_at_ms = now_ms;

    if health_tracker.should_skip_lead(
        ProviderId::BraveWebSearch.as_str(),
        now_ms,
        config.health_policy,
    ) {
        if !fallback_enabled {
            return Err(ProviderError {
                provider_id: ProviderId::WebProviderLadder,
                kind: ProviderErrorKind::PolicyViolation,
                status_code: None,
                message: "fallback_provider_disabled".to_string(),
                latency_ms: 0,
            });
        }
        fallback_used = true;
        let skipped_transitions = default_degraded_transitions(request.created_at_ms);
        let skipped_debug_packet = try_build_debug_packet(DebugPacketContext {
            trace_id: request.trace_id.as_str(),
            status: DebugStatus::Degraded,
            provider: "BraveWebSearch",
            error_kind: "health_cooldown",
            reason_code: "provider_upstream_failed",
            proxy_mode: None,
            source_url: None,
            created_at_ms: request.created_at_ms,
            turn_state_transitions: &skipped_transitions,
            debug_hint: Some("lead provider skipped due to cooldown"),
            fallback_used: Some(true),
            health_status_before_fallback: Some(health_to_debug_status(brave_state)),
        })
        .ok()
        .and_then(|packet| serde_json::to_value(packet).ok())
        .unwrap_or(Value::Null);

        provider_runs.push(json!({
            "provider_id": ProviderId::BraveWebSearch.as_str(),
            "endpoint": "web",
            "latency_ms": 0,
            "error": Value::Null,
            "triggered_fallback": true,
            "fallback_trigger": "health_cooldown",
            "health_state": brave_state.as_str(),
            "skipped_due_to_cooldown": true,
            "debug_packet": skipped_debug_packet,
        }));

        provider_call_budget
            .record_fallback_call()
            .map_err(|reason| budget_exhausted_error(reason))?;
        if let Some(blocked) =
            stage8_web_provider_block(&request, config, ProviderControlProvider::OpenAiWebSearch)
        {
            return Err(stage2_provider_error(ProviderId::OpenAiWebSearch, blocked));
        }
        let (openai, cache_hit) = run_provider_call_with_cache(
            &mut l1_cache,
            cache_enabled,
            &request,
            ProviderId::OpenAiWebSearch,
            CacheMode::Web,
            cache_policy_snapshot_id.as_str(),
            now_ms,
            || {
                let openai_key = resolve_openai_api_key(config)?;
                openai_fallback::execute_openai_web_search(
                    &config.openai_endpoint,
                    &openai_key,
                    &config.openai_model,
                    &request.query,
                    effective_max_results,
                    effective_timeout_ms,
                    &config.user_agent,
                    &config.proxy_config,
                )
            },
        )?;
        if let Some(hit) = cache_hit {
            evidence_retrieved_at_ms = evidence_retrieved_at_ms.min(hit.retrieved_at_ms);
        }
        health_tracker.record_success(ProviderId::OpenAiWebSearch.as_str());
        results_count_fallback = openai.results.len();
        openai_results = openai.results;
        provider_runs.push(provider_run_success(
            ProviderId::OpenAiWebSearch,
            openai.latency_ms,
            true,
            Some("health_cooldown"),
            health_tracker.snapshot(
                ProviderId::OpenAiWebSearch.as_str(),
                now_ms,
                config.health_policy,
            ),
        ));
    } else {
        lead_attempted = true;
        provider_call_budget
            .record_lead_call()
            .map_err(|reason| budget_exhausted_error(reason))?;
        match run_provider_call_with_cache(
            &mut l1_cache,
            cache_enabled,
            &request,
            ProviderId::BraveWebSearch,
            CacheMode::Web,
            cache_policy_snapshot_id.as_str(),
            now_ms,
            || {
                let brave_key = resolve_brave_api_key(config)?;
                brave_adapter::execute_brave_web_search(
                    &config.brave_endpoint,
                    &brave_key,
                    &request.query,
                    effective_max_results,
                    effective_timeout_ms,
                    &config.user_agent,
                    &config.proxy_config,
                )
            },
        ) {
            Ok(brave) => {
                let (brave, cache_hit) = brave;
                if let Some(hit) = cache_hit {
                    evidence_retrieved_at_ms = evidence_retrieved_at_ms.min(hit.retrieved_at_ms);
                }
                health_tracker.record_success(ProviderId::BraveWebSearch.as_str());
                results_count_lead = brave.results.len();
                brave_results = brave.results;
                provider_runs.push(provider_run_success(
                    ProviderId::BraveWebSearch,
                    brave.latency_ms,
                    false,
                    None,
                    health_tracker.snapshot(
                        ProviderId::BraveWebSearch.as_str(),
                        now_ms,
                        config.health_policy,
                    ),
                ));
            }
            Err(brave_err) => {
                health_tracker.record_failure(
                    ProviderId::BraveWebSearch.as_str(),
                    now_ms,
                    config.health_policy,
                );

                let fallback_trigger = fallback_trigger_label(brave_err.kind);
                let should_fallback = fallback_enabled && should_trigger_fallback(brave_err.kind);
                if should_fallback {
                    fallback_used = true;
                    fallback_reason_code = Some(brave_err.reason_code().to_string());
                }

                provider_runs.push(provider_run_error(
                    &brave_err,
                    should_fallback,
                    fallback_trigger,
                    health_tracker.snapshot(
                        ProviderId::BraveWebSearch.as_str(),
                        now_ms,
                        config.health_policy,
                    ),
                    request.trace_id.as_str(),
                    request.created_at_ms,
                ));

                if should_fallback {
                    provider_call_budget
                        .record_fallback_call()
                        .map_err(|reason| budget_exhausted_error(reason))?;
                    if let Some(blocked) = stage8_web_provider_block(
                        &request,
                        config,
                        ProviderControlProvider::OpenAiWebSearch,
                    ) {
                        return Err(stage2_provider_error(ProviderId::OpenAiWebSearch, blocked));
                    }
                    match run_provider_call_with_cache(
                        &mut l1_cache,
                        cache_enabled,
                        &request,
                        ProviderId::OpenAiWebSearch,
                        CacheMode::Web,
                        cache_policy_snapshot_id.as_str(),
                        now_ms,
                        || {
                            let openai_key = resolve_openai_api_key(config)?;
                            openai_fallback::execute_openai_web_search(
                                &config.openai_endpoint,
                                &openai_key,
                                &config.openai_model,
                                &request.query,
                                effective_max_results,
                                effective_timeout_ms,
                                &config.user_agent,
                                &config.proxy_config,
                            )
                        },
                    ) {
                        Ok(openai) => {
                            let (openai, cache_hit) = openai;
                            if let Some(hit) = cache_hit {
                                evidence_retrieved_at_ms =
                                    evidence_retrieved_at_ms.min(hit.retrieved_at_ms);
                            }
                            health_tracker.record_success(ProviderId::OpenAiWebSearch.as_str());
                            results_count_fallback = openai.results.len();
                            openai_results = openai.results;
                            provider_runs.push(provider_run_success(
                                ProviderId::OpenAiWebSearch,
                                openai.latency_ms,
                                true,
                                fallback_trigger,
                                health_tracker.snapshot(
                                    ProviderId::OpenAiWebSearch.as_str(),
                                    now_ms,
                                    config.health_policy,
                                ),
                            ));
                        }
                        Err(openai_err) => {
                            health_tracker.record_failure(
                                ProviderId::OpenAiWebSearch.as_str(),
                                now_ms,
                                config.health_policy,
                            );
                            provider_runs.push(provider_run_error(
                                &openai_err,
                                false,
                                fallback_trigger,
                                health_tracker.snapshot(
                                    ProviderId::OpenAiWebSearch.as_str(),
                                    now_ms,
                                    config.health_policy,
                                ),
                                request.trace_id.as_str(),
                                request.created_at_ms,
                            ));
                            return Err(openai_err);
                        }
                    }
                } else {
                    return Err(brave_err);
                }
            }
        }
    }

    let merged = merge_results(&brave_results, &openai_results);
    if merged.merged_results.is_empty() {
        let error = ProviderError {
            provider_id: ProviderId::WebProviderLadder,
            kind: ProviderErrorKind::EmptyResults,
            status_code: None,
            message: "provider ladder produced zero merged results".to_string(),
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

    let sources: Vec<Value> = merged
        .merged_results
        .iter()
        .enumerate()
        .map(|(index, result)| {
            json!({
                "title": result.title,
                "url": result.url,
                "snippet": result.snippet,
                "media_type": "web",
                "provider_id": result.provider_id.as_str(),
                "rank": index + 1,
                "canonical_url": result.canonical_url,
            })
        })
        .collect();

    let content_chunks = build_content_chunks(&merged.merged_results)?;

    let provider_health_snapshot = json!({
        "brave": health_tracker
            .snapshot(ProviderId::BraveWebSearch.as_str(), now_ms, config.health_policy)
            .as_str(),
        "openai": health_tracker
            .snapshot(ProviderId::OpenAiWebSearch.as_str(), now_ms, config.health_policy)
            .as_str(),
    });

    let evidence_packet = json!({
        "schema_version": "1.0.0",
        "produced_by": WEB_PROVIDER_ENGINE_ID,
        "intended_consumers": request.intended_consumers,
        "created_at_ms": request.created_at_ms,
        "trace_id": request.trace_id,
        "query": request.query,
        "retrieved_at_ms": evidence_retrieved_at_ms,
        "provider_runs": provider_runs,
        "sources": sources,
        "content_chunks": content_chunks,
        "trust_metadata": {
            "provider_ladder": {
                "lead_provider": ProviderId::BraveWebSearch.as_str(),
                "fallback_provider": ProviderId::OpenAiWebSearch.as_str(),
                "fallback_used": fallback_used,
                "dedup_count": merged.dedup_count,
                "health_snapshot": provider_health_snapshot,
                "importance_tier": request.importance_tier.as_str(),
                "max_total_provider_calls_per_turn": tier_caps.max_total_provider_calls_per_turn,
                "max_fallback_invocations_per_turn": tier_caps.max_fallback_invocations_per_turn,
                "max_retries_per_provider": tier_caps.max_retries_per_provider,
                "total_provider_calls": provider_call_budget.total_provider_calls(),
                "fallback_invocations": provider_call_budget.fallback_invocations(),
                "timeout_per_provider_ms": effective_timeout_ms,
            }
        }
    });

    let audit_metrics = WebProviderAuditMetrics {
        lead_attempted,
        fallback_used,
        fallback_reason_code,
        provider_health_snapshot,
        results_count_lead,
        results_count_fallback,
        dedup_count: merged.dedup_count,
    };

    Ok(WebProviderLadderResult {
        evidence_packet,
        audit_metrics,
    })
}

fn stage2_web_provider_block(
    request: &ParsedToolRequest,
    config: &WebProviderRuntimeConfig,
) -> Option<ProviderGateDecision> {
    stage8_web_provider_block(request, config, ProviderControlProvider::BraveWebSearch)
}

fn stage8_web_provider_block(
    request: &ParsedToolRequest,
    config: &WebProviderRuntimeConfig,
    provider: ProviderControlProvider,
) -> Option<ProviderGateDecision> {
    let fake_endpoint = is_local_fake_endpoint(&config.brave_endpoint)
        && is_local_fake_endpoint(&config.openai_endpoint);
    let decision = if fake_endpoint {
        fake_provider_decision(ProviderControlRoute::WebSearch, provider, &request.query, 1)
    } else {
        disabled_provider_decision(ProviderControlRoute::WebSearch, provider, &request.query)
    };
    (!decision.allowed).then_some(decision)
}

fn stage2_provider_error(provider_id: ProviderId, decision: ProviderGateDecision) -> ProviderError {
    ProviderError {
        provider_id,
        kind: ProviderErrorKind::PolicyViolation,
        status_code: None,
        message: decision.disabled_trace_line(),
        latency_ms: 0,
    }
}

fn build_content_chunks(results: &[NormalizedSearchResult]) -> Result<Vec<Value>, ProviderError> {
    let mut chunks = Vec::new();
    let mut seen_chunk_text: BTreeMap<String, String> = BTreeMap::new();

    for (index, result) in results.iter().enumerate() {
        let normalized_text = normalize_document_for_chunking(&result.snippet);
        if normalized_text.is_empty() {
            continue;
        }

        let text_chunk = TextChunk {
            chunk_index: index,
            normalized_text: normalized_text.clone(),
            norm_version: NORM_VERSION,
            chunk_version: CHUNK_VERSION,
        };

        let chunk_id = derive_chunk_id(&result.canonical_url, &text_chunk, &Sha256ChunkHasher);

        if let Some(previous) = seen_chunk_text.get(chunk_id.as_str()) {
            if previous != &normalized_text {
                return Err(ProviderError {
                    provider_id: ProviderId::WebProviderLadder,
                    kind: ProviderErrorKind::HashCollisionDetected,
                    status_code: None,
                    message: format!("chunk_id collision detected for {}", chunk_id),
                    latency_ms: 0,
                });
            }
        } else {
            seen_chunk_text.insert(chunk_id.clone(), normalized_text.clone());
        }

        let source_url = result.url.clone();
        let canonical_url = result.canonical_url.clone();
        chunks.push(json!({
            "chunk_id": chunk_id,
            "hash_version": HASH_VERSION,
            "norm_version": NORM_VERSION,
            "chunk_version": CHUNK_VERSION,
            "source_url": source_url,
            "canonical_url": canonical_url,
            "chunk_index": index,
            "text_excerpt": bounded_excerpt(&normalized_text, 320),
            "text_len_chars": normalized_text.chars().count(),
            "citation": {
                "chunk_id": chunk_id,
                "source_url": result.url,
            }
        }));
    }

    Ok(chunks)
}

fn resolve_brave_api_key(config: &WebProviderRuntimeConfig) -> Result<String, ProviderError> {
    config
        .brave_api_key_override
        .clone()
        .or_else(|| resolve_secret_from_vault(ProviderSecretId::BraveSearchApiKey))
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| ProviderError {
            provider_id: ProviderId::BraveWebSearch,
            kind: ProviderErrorKind::ProviderUnconfigured,
            status_code: None,
            message: "missing brave_search_api_key in device vault".to_string(),
            latency_ms: 0,
        })
}

fn resolve_openai_api_key(config: &WebProviderRuntimeConfig) -> Result<String, ProviderError> {
    config
        .openai_api_key_override
        .clone()
        .or_else(|| resolve_secret_from_vault(ProviderSecretId::OpenAIApiKey))
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| ProviderError {
            provider_id: ProviderId::OpenAiWebSearch,
            kind: ProviderErrorKind::ProviderUnconfigured,
            status_code: None,
            message: "missing openai_api_key in device vault".to_string(),
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

fn run_provider_call_with_cache<F>(
    l1_cache: &mut L1Cache,
    cache_enabled: bool,
    request: &ParsedToolRequest,
    provider_id: ProviderId,
    mode: CacheMode,
    policy_snapshot_id: &str,
    now_ms: i64,
    call: F,
) -> Result<
    (
        ProviderCallSuccess,
        Option<CacheLookupHit<ProviderCallSuccess>>,
    ),
    ProviderError,
>
where
    F: FnOnce() -> Result<ProviderCallSuccess, ProviderError>,
{
    if cache_enabled {
        let key = CacheKey::new(
            mode,
            request.query.as_str(),
            None,
            Some(provider_id.as_str()),
            request.importance_tier,
            Some(policy_snapshot_id),
        );
        match lookup_typed::<ProviderCallSuccess>(
            l1_cache,
            &key,
            now_ms,
            CACHE_SCHEMA_VERSION,
            policy_snapshot_id,
        ) {
            Ok(Some(hit)) => return Ok((hit.value.clone(), Some(hit))),
            Ok(None) => {}
            Err(err) => {
                return Err(ProviderError {
                    provider_id,
                    kind: ProviderErrorKind::PolicyViolation,
                    status_code: None,
                    message: format!("cache lookup rejected: {}", err),
                    latency_ms: 0,
                })
            }
        }

        let fresh = call()?;
        let ttl_ms = ttl_ms_for(mode, request.importance_tier);
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
        .map_err(|err| ProviderError {
            provider_id,
            kind: ProviderErrorKind::PolicyViolation,
            status_code: None,
            message: format!("cache store rejected: {}", err),
            latency_ms: 0,
        })?;
        return Ok((fresh, None));
    }

    call().map(|fresh| (fresh, None))
}

fn web_cache_enabled() -> bool {
    match std::env::var("SELENE_WEB_CACHE_ENABLED") {
        Ok(value) => matches!(
            value.trim().to_ascii_lowercase().as_str(),
            "1" | "true" | "yes" | "on"
        ),
        Err(_) => false,
    }
}

fn cache_policy_snapshot_id() -> String {
    std::env::var("SELENE_WEB_CACHE_POLICY_SNAPSHOT_ID")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| DEFAULT_CACHE_POLICY_SNAPSHOT_ID.to_string())
}

fn budget_exhausted_error(message: &str) -> ProviderError {
    ProviderError {
        provider_id: ProviderId::WebProviderLadder,
        kind: ProviderErrorKind::BudgetExhausted,
        status_code: None,
        message: format!("provider call budget exhausted: {}", message),
        latency_ms: 0,
    }
}

fn provider_run_success(
    provider_id: ProviderId,
    latency_ms: u64,
    triggered_fallback: bool,
    fallback_trigger: Option<&str>,
    health_state: ProviderHealthState,
) -> Value {
    json!({
        "provider_id": provider_id.as_str(),
        "endpoint": "web",
        "latency_ms": latency_ms,
        "error": Value::Null,
        "triggered_fallback": triggered_fallback,
        "fallback_trigger": fallback_trigger,
        "health_state": health_state.as_str(),
    })
}

fn provider_run_error(
    error: &ProviderError,
    triggered_fallback: bool,
    fallback_trigger: Option<&str>,
    health_state: ProviderHealthState,
    trace_id: &str,
    created_at_ms: i64,
) -> Value {
    let status = if triggered_fallback {
        DebugStatus::Degraded
    } else {
        DebugStatus::Failed
    };
    let transitions = if triggered_fallback {
        default_degraded_transitions(created_at_ms)
    } else {
        default_failed_transitions(created_at_ms)
    };
    let reason_code = error.reason_code();
    let debug_packet = try_build_debug_packet(DebugPacketContext {
        trace_id,
        status,
        provider: provider_name_for_debug(error.provider_id),
        error_kind: error.kind.as_str(),
        reason_code,
        proxy_mode: None,
        source_url: None,
        created_at_ms,
        turn_state_transitions: &transitions,
        debug_hint: Some(error.message.as_str()),
        fallback_used: Some(triggered_fallback),
        health_status_before_fallback: Some(health_to_debug_status(health_state)),
    })
    .ok()
    .and_then(|packet| serde_json::to_value(packet).ok())
    .unwrap_or(Value::Null);

    json!({
        "provider_id": error.provider_id.as_str(),
        "endpoint": "web",
        "latency_ms": error.latency_ms,
        "error": {
            "error_kind": error.kind.as_str(),
            "reason_code": error.reason_code(),
            "status_code": error.status_code,
            "message": error.message,
            "debug_packet": debug_packet,
        },
        "triggered_fallback": triggered_fallback,
        "fallback_trigger": fallback_trigger,
        "health_state": health_state.as_str(),
    })
}

fn provider_name_for_debug(provider_id: ProviderId) -> &'static str {
    match provider_id {
        ProviderId::WebProviderLadder => "Planning",
        ProviderId::BraveWebSearch => "BraveWebSearch",
        ProviderId::OpenAiWebSearch => "OpenAI_WebSearch",
    }
}

fn health_to_debug_status(state: ProviderHealthState) -> HealthStatusBeforeFallback {
    match state {
        ProviderHealthState::Healthy => HealthStatusBeforeFallback::Healthy,
        ProviderHealthState::Degraded => HealthStatusBeforeFallback::Degraded,
        ProviderHealthState::Cooldown => HealthStatusBeforeFallback::Cooldown,
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

#[cfg(test)]
pub mod web_provider_tests;
