#![forbid(unsafe_code)]

use crate::web_search_plan::proxy::proxy_redaction::redact_proxy_url;
use crate::web_search_plan::proxy::proxy_self_check::run_startup_self_check;
use crate::web_search_plan::proxy::ProxyErrorKind;
use crate::web_search_plan::url::canonical::canonicalize_url;
use crate::web_search_plan::web_provider::{
    normalize_text_value, NormalizedSearchResult, ProviderCallSuccess, ProviderError,
    ProviderErrorKind, ProviderId,
};
use serde_json::Value;
use std::time::{Duration, Instant};

pub fn execute_brave_web_search(
    endpoint: &str,
    api_key: &str,
    query: &str,
    max_results: usize,
    timeout_ms: u64,
    user_agent: &str,
    proxy_config: &crate::web_search_plan::proxy::proxy_config::ProxyConfig,
) -> Result<ProviderCallSuccess, ProviderError> {
    let start = Instant::now();
    let mut builder = ureq::AgentBuilder::new()
        .timeout_connect(Duration::from_millis(timeout_ms))
        .timeout_read(Duration::from_millis(timeout_ms))
        .timeout_write(Duration::from_millis(timeout_ms))
        .user_agent(user_agent)
        .try_proxy_from_env(false);

    if let Err(check) = run_startup_self_check(proxy_config) {
        if check.error_kind == ProxyErrorKind::ProxyMisconfigured
            && check.severity.as_str() == "critical"
        {
            return Err(ProviderError {
                provider_id: ProviderId::BraveWebSearch,
                kind: ProviderErrorKind::ProxyMisconfigured,
                status_code: None,
                message: check.details,
                latency_ms: start.elapsed().as_millis() as u64,
            });
        }
    }

    if let Some(proxy_raw) = select_proxy_url(endpoint, proxy_config) {
        let _ = redact_proxy_url(proxy_raw).map_err(|_| ProviderError {
            provider_id: ProviderId::BraveWebSearch,
            kind: ProviderErrorKind::ProxyMisconfigured,
            status_code: None,
            message: "proxy URL redaction failed".to_string(),
            latency_ms: start.elapsed().as_millis() as u64,
        })?;
        let proxy = ureq::Proxy::new(proxy_raw).map_err(|_| ProviderError {
            provider_id: ProviderId::BraveWebSearch,
            kind: ProviderErrorKind::ProxyMisconfigured,
            status_code: None,
            message: "invalid proxy URL".to_string(),
            latency_ms: start.elapsed().as_millis() as u64,
        })?;
        builder = builder.proxy(proxy);
    }

    let agent = builder.build();
    let response = agent
        .get(endpoint)
        .set("Accept", "application/json")
        .set("X-Subscription-Token", api_key)
        .query("q", query)
        .query("count", &max_results.to_string())
        .call()
        .map_err(|err| {
            map_ureq_error(
                ProviderId::BraveWebSearch,
                err,
                start.elapsed().as_millis() as u64,
            )
        })?;

    let body: Value =
        serde_json::from_reader(response.into_reader()).map_err(|_| ProviderError {
            provider_id: ProviderId::BraveWebSearch,
            kind: ProviderErrorKind::ParseFailed,
            status_code: None,
            message: "brave JSON parse failed".to_string(),
            latency_ms: start.elapsed().as_millis() as u64,
        })?;

    let mut results = Vec::new();
    for item in candidate_results(&body) {
        if results.len() >= max_results {
            break;
        }

        let Some(url) = item
            .get("url")
            .or_else(|| item.get("link"))
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|url| !url.is_empty())
        else {
            continue;
        };

        let canonical = canonicalize_url(url).map_err(|_| ProviderError {
            provider_id: ProviderId::BraveWebSearch,
            kind: ProviderErrorKind::ParseFailed,
            status_code: None,
            message: format!("brave result URL is not canonicalizable: {}", url),
            latency_ms: start.elapsed().as_millis() as u64,
        })?;

        let title = item
            .get("title")
            .or_else(|| item.get("name"))
            .and_then(Value::as_str)
            .map(normalize_text_value)
            .filter(|v| !v.is_empty())
            .unwrap_or_else(|| "Result".to_string());

        let snippet = item
            .get("description")
            .or_else(|| item.get("snippet"))
            .or_else(|| {
                item.get("extra_snippets")
                    .and_then(Value::as_array)
                    .and_then(|arr| arr.first())
            })
            .and_then(Value::as_str)
            .map(normalize_text_value)
            .filter(|v| !v.is_empty())
            .unwrap_or_else(|| "No snippet available.".to_string());

        results.push(NormalizedSearchResult {
            title,
            url: url.to_string(),
            snippet,
            canonical_url: canonical.canonical_url,
            citation_url: url.to_string(),
            provider_id: ProviderId::BraveWebSearch,
            provider_rank: results.len() + 1,
        });
    }

    if results.is_empty() {
        return Err(ProviderError {
            provider_id: ProviderId::BraveWebSearch,
            kind: ProviderErrorKind::EmptyResults,
            status_code: None,
            message: "brave returned zero usable results".to_string(),
            latency_ms: start.elapsed().as_millis() as u64,
        });
    }

    Ok(ProviderCallSuccess {
        results,
        latency_ms: start.elapsed().as_millis() as u64,
    })
}

fn candidate_results(body: &Value) -> Vec<&Value> {
    let mut out = Vec::new();
    if let Some(items) = body.pointer("/web/results").and_then(Value::as_array) {
        out.extend(items.iter());
    }
    if let Some(items) = body.pointer("/results").and_then(Value::as_array) {
        out.extend(items.iter());
    }
    out
}

fn select_proxy_url<'a>(
    endpoint: &str,
    config: &'a crate::web_search_plan::proxy::proxy_config::ProxyConfig,
) -> Option<&'a str> {
    let is_https = endpoint.trim().to_ascii_lowercase().starts_with("https://");
    if is_https {
        config.https_proxy_url.as_deref()
    } else {
        config.http_proxy_url.as_deref()
    }
}

fn map_ureq_error(provider_id: ProviderId, err: ureq::Error, latency_ms: u64) -> ProviderError {
    match err {
        ureq::Error::Status(status, _) => ProviderError {
            provider_id,
            kind: ProviderErrorKind::HttpNon200,
            status_code: Some(status as u16),
            message: format!("provider returned HTTP status {}", status),
            latency_ms,
        },
        ureq::Error::Transport(transport) => {
            let combined = format!("{:?} {}", transport.kind(), transport).to_ascii_lowercase();
            let kind = if combined.contains("timeout") {
                ProviderErrorKind::TimeoutExceeded
            } else if combined.contains("tls") || combined.contains("ssl") {
                ProviderErrorKind::TlsFailed
            } else if combined.contains("dns") {
                ProviderErrorKind::DnsFailed
            } else if combined.contains("connect") || combined.contains("connection") {
                ProviderErrorKind::ConnectFailed
            } else {
                ProviderErrorKind::TransportFailed
            };

            ProviderError {
                provider_id,
                kind,
                status_code: None,
                message: "transport failure during provider call".to_string(),
                latency_ms,
            }
        }
    }
}
