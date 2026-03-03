#![forbid(unsafe_code)]

pub mod brave_adapter;
pub mod fallback_policy;
pub mod health_state;
pub mod openai_fallback;
pub mod provider_merge;

use crate::web_search_plan::chunk::bounded_excerpt;
use crate::web_search_plan::chunk::chunker::{TextChunk, CHUNK_VERSION};
use crate::web_search_plan::chunk::hasher::{derive_chunk_id, Sha256ChunkHasher, HASH_VERSION};
use crate::web_search_plan::chunk::normalize::{normalize_document_for_chunking, NORM_VERSION};
use crate::web_search_plan::proxy::proxy_config::{ProxyConfig, SystemEnvProvider};
use crate::web_search_plan::proxy::ProxyMode;
use crate::web_search_plan::web_provider::fallback_policy::{
    fallback_trigger_label, should_trigger_fallback,
};
use crate::web_search_plan::web_provider::health_state::{
    HealthPolicy, ProviderHealthState, ProviderHealthTracker,
};
use crate::web_search_plan::web_provider::provider_merge::merge_results;
use selene_kernel_contracts::provider_secrets::ProviderSecretId;
use serde_json::{json, Map, Value};
use std::collections::BTreeMap;

pub const DEFAULT_BRAVE_WEB_ENDPOINT: &str = "https://api.search.brave.com/res/v1/web/search";
pub const DEFAULT_OPENAI_RESPONSES_ENDPOINT: &str = "https://api.openai.com/v1/responses";
pub const DEFAULT_OPENAI_WEB_MODEL: &str = "gpt-4o-mini";
pub const DEFAULT_MAX_RESULTS: usize = 5;
pub const DEFAULT_TIMEOUT_MS: u64 = 2_500;
pub const DEFAULT_USER_AGENT: &str = "selene-web-provider-ladder/1.0";
pub const WEB_PROVIDER_ENGINE_ID: &str = "PH1.E";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NormalizedSearchResult {
    pub title: String,
    pub url: String,
    pub snippet: String,
    pub canonical_url: String,
    pub citation_url: String,
    pub provider_id: ProviderId,
    pub provider_rank: usize,
}

#[derive(Debug, Clone)]
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
    })
}

fn execute_web_provider_ladder(
    request: ParsedToolRequest,
    now_ms: i64,
    health_tracker: &mut ProviderHealthTracker,
    config: &WebProviderRuntimeConfig,
) -> Result<WebProviderLadderResult, ProviderError> {
    let brave_state = health_tracker.snapshot(
        ProviderId::BraveWebSearch.as_str(),
        now_ms,
        config.health_policy,
    );
    let mut lead_attempted = false;
    let mut fallback_used = false;
    let mut fallback_reason_code = None;

    let mut provider_runs = Vec::new();
    let mut brave_results = Vec::new();
    let mut openai_results = Vec::new();
    let mut results_count_lead = 0usize;
    let mut results_count_fallback = 0usize;

    if health_tracker.should_skip_lead(
        ProviderId::BraveWebSearch.as_str(),
        now_ms,
        config.health_policy,
    ) {
        fallback_used = true;
        provider_runs.push(json!({
            "provider_id": ProviderId::BraveWebSearch.as_str(),
            "endpoint": "web",
            "latency_ms": 0,
            "error": Value::Null,
            "triggered_fallback": true,
            "fallback_trigger": "health_cooldown",
            "health_state": brave_state.as_str(),
            "skipped_due_to_cooldown": true,
        }));

        let openai_key = resolve_openai_api_key(config)?;
        let openai = openai_fallback::execute_openai_web_search(
            &config.openai_endpoint,
            &openai_key,
            &config.openai_model,
            &request.query,
            config.max_results,
            config.timeout_ms,
            &config.user_agent,
            &config.proxy_config,
        )?;
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
        let brave_key = resolve_brave_api_key(config)?;
        match brave_adapter::execute_brave_web_search(
            &config.brave_endpoint,
            &brave_key,
            &request.query,
            config.max_results,
            config.timeout_ms,
            &config.user_agent,
            &config.proxy_config,
        ) {
            Ok(brave) => {
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
                let should_fallback = should_trigger_fallback(brave_err.kind);
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
                ));

                if should_fallback {
                    let openai_key = resolve_openai_api_key(config)?;
                    match openai_fallback::execute_openai_web_search(
                        &config.openai_endpoint,
                        &openai_key,
                        &config.openai_model,
                        &request.query,
                        config.max_results,
                        config.timeout_ms,
                        &config.user_agent,
                        &config.proxy_config,
                    ) {
                        Ok(openai) => {
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
        return Err(ProviderError {
            provider_id: ProviderId::WebProviderLadder,
            kind: ProviderErrorKind::EmptyResults,
            status_code: None,
            message: "provider ladder produced zero merged results".to_string(),
            latency_ms: 0,
        });
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
        "retrieved_at_ms": now_ms,
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
) -> Value {
    json!({
        "provider_id": error.provider_id.as_str(),
        "endpoint": "web",
        "latency_ms": error.latency_ms,
        "error": {
            "error_kind": error.kind.as_str(),
            "reason_code": error.reason_code(),
            "status_code": error.status_code,
            "message": error.message,
        },
        "triggered_fallback": triggered_fallback,
        "fallback_trigger": fallback_trigger,
        "health_state": health_state.as_str(),
    })
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
