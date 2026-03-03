#![forbid(unsafe_code)]

pub mod brave_news_adapter;
pub mod conflict_cluster;
pub mod diversity;
pub mod gdelt_adapter;
pub mod merge;
pub mod recency_policy;

use crate::web_search_plan::chunk::bounded_excerpt;
use crate::web_search_plan::chunk::chunker::{TextChunk, CHUNK_VERSION};
use crate::web_search_plan::chunk::hasher::{derive_chunk_id, Sha256ChunkHasher, HASH_VERSION};
use crate::web_search_plan::chunk::normalize::{normalize_document_for_chunking, NORM_VERSION};
use crate::web_search_plan::news_provider::conflict_cluster::{
    build_conflict_clusters, cluster_lookup_by_canonical_url,
};
use crate::web_search_plan::news_provider::diversity::{
    distinct_domain_count, diversity_threshold_met,
};
use crate::web_search_plan::news_provider::merge::merge_news_results;
use crate::web_search_plan::news_provider::recency_policy::{
    freshness_score, normalize_published_at, recency_window_days, within_recency_window,
    ImportanceTier,
};
use crate::web_search_plan::proxy::proxy_config::{ProxyConfig, SystemEnvProvider};
use crate::web_search_plan::proxy::ProxyMode;
use selene_kernel_contracts::provider_secrets::ProviderSecretId;
use serde_json::{json, Map, Value};
use std::collections::BTreeMap;

pub const DEFAULT_BRAVE_NEWS_ENDPOINT: &str = "https://api.search.brave.com/res/v1/news/search";
pub const DEFAULT_GDELT_ENDPOINT: &str = "https://api.gdeltproject.org/api/v2/doc/doc";
pub const DEFAULT_TIMEOUT_MS: u64 = 2_500;
pub const DEFAULT_MAX_RESULTS: usize = 6;
pub const DEFAULT_USER_AGENT: &str = "selene-news-provider-ladder/1.0";
pub const NEWS_PROVIDER_ENGINE_ID: &str = "PH1.E";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
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

#[derive(Debug, Clone, PartialEq, Eq)]
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

#[derive(Debug, Clone, PartialEq)]
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

pub fn execute_news_provider_ladder_from_tool_request(
    tool_request_packet: &Value,
    now_ms: i64,
    config: &NewsRuntimeConfig,
) -> Result<NewsProviderLadderResult, NewsProviderError> {
    let request = parse_tool_request(tool_request_packet)?;
    execute_news_provider_ladder(request, now_ms, config)
}

pub fn append_news_provider_audit_fields(
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
        "news_provider_audit".to_string(),
        json!({
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

fn execute_news_provider_ladder(
    request: ParsedToolRequest,
    now_ms: i64,
    config: &NewsRuntimeConfig,
) -> Result<NewsProviderLadderResult, NewsProviderError> {
    let window_days = recency_window_days(request.importance_tier);

    let mut provider_runs = Vec::new();
    let mut assist_used = false;
    let mut filtered_total = 0usize;

    let mut brave_filtered = Vec::new();
    let mut gdelt_filtered = Vec::new();

    match resolve_brave_api_key(config) {
        Ok(brave_key) => match brave_news_adapter::execute_brave_news_search(
            &config.brave_news_endpoint,
            &brave_key,
            &request.query,
            config.max_results,
            config.timeout_ms,
            &config.user_agent,
            &config.proxy_config,
        ) {
            Ok(success) => {
                let (normalized, filtered_count) =
                    normalize_and_filter_results(&success.results, now_ms, window_days);
                filtered_total = filtered_total.saturating_add(filtered_count);
                brave_filtered = normalized;
                provider_runs.push(provider_run_success(
                    NewsProviderId::BraveNewsSearch,
                    success.latency_ms,
                    brave_filtered.len(),
                ));
            }
            Err(err) => {
                provider_runs.push(provider_run_error(&err, 0));
            }
        },
        Err(err) => {
            provider_runs.push(provider_run_error(&err, 0));
        }
    }

    let brave_domain_count = distinct_domain_count(&brave_filtered);
    let needs_assist = brave_filtered.is_empty()
        || (request.importance_tier == ImportanceTier::High
            && !diversity_threshold_met(request.importance_tier, brave_domain_count));

    if needs_assist {
        assist_used = true;
        match gdelt_adapter::execute_gdelt_news_search(
            &config.gdelt_endpoint,
            &request.query,
            config.max_results,
            config.timeout_ms,
            &config.user_agent,
            &config.proxy_config,
        ) {
            Ok(success) => {
                let (normalized, filtered_count) =
                    normalize_and_filter_results(&success.results, now_ms, window_days);
                filtered_total = filtered_total.saturating_add(filtered_count);
                gdelt_filtered = normalized;
                provider_runs.push(provider_run_success(
                    NewsProviderId::GdeltNewsAssist,
                    success.latency_ms,
                    gdelt_filtered.len(),
                ));
            }
            Err(err) => {
                provider_runs.push(provider_run_error(&err, 0));
                if brave_filtered.is_empty() {
                    return Err(err);
                }
            }
        }
    }

    let merge = merge_news_results(&brave_filtered, &gdelt_filtered);
    if merge.merged_results.is_empty() {
        return Err(NewsProviderError {
            provider_id: NewsProviderId::NewsProviderLadder,
            kind: NewsProviderErrorKind::EmptyResults,
            status_code: None,
            message: "no news results after recency filtering and merge".to_string(),
            latency_ms: 0,
        });
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
                "source_urls": cluster.source_urls.clone(),
                "claims": cluster.claims.clone(),
            })
        })
        .collect();

    let evidence_packet = json!({
        "schema_version": "1.0.0",
        "produced_by": NEWS_PROVIDER_ENGINE_ID,
        "intended_consumers": request.intended_consumers,
        "created_at_ms": request.created_at_ms,
        "trace_id": request.trace_id,
        "query": request.query,
        "retrieved_at_ms": now_ms,
        "provider_runs": provider_runs,
        "sources": sources,
        "content_chunks": content_chunks,
        "trust_metadata": {
            "news_provider_ladder": {
                "lead_provider": NewsProviderId::BraveNewsSearch.as_str(),
                "assist_provider": NewsProviderId::GdeltNewsAssist.as_str(),
                "assist_used": assist_used,
                "recency_window_days": window_days,
                "filtered_by_recency_count": filtered_total,
                "distinct_domain_count": domain_count,
                "diversity_threshold_met": diversity_met,
                "dedup_count": merge.dedup_count,
                "corroborated_count": merge.corroborated_canonical_urls.len(),
                "contradiction_clusters": trust_metadata_clusters,
            }
        }
    });

    let audit_metrics = NewsAuditMetrics {
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
) -> Value {
    json!({
        "provider_id": provider_id.as_str(),
        "endpoint": "news",
        "latency_ms": latency_ms,
        "results_count": results_count,
        "error": Value::Null,
    })
}

fn provider_run_error(error: &NewsProviderError, results_count: usize) -> Value {
    json!({
        "provider_id": error.provider_id.as_str(),
        "endpoint": "news",
        "latency_ms": error.latency_ms,
        "results_count": results_count,
        "error": {
            "error_kind": error.kind.as_str(),
            "reason_code": error.reason_code(),
            "status_code": error.status_code,
            "message": error.message,
        }
    })
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
