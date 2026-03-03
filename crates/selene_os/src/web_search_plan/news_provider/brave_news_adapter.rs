#![forbid(unsafe_code)]

use crate::web_search_plan::news_provider::{
    extract_domain, normalize_text_value, NewsProviderError, NewsProviderErrorKind, NewsProviderId,
    ProviderNewsItem,
};
use crate::web_search_plan::proxy::proxy_redaction::redact_proxy_url;
use crate::web_search_plan::proxy::proxy_self_check::run_startup_self_check;
use crate::web_search_plan::proxy::ProxyErrorKind;
use crate::web_search_plan::url::canonical::canonicalize_url;
use serde_json::Value;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct ProviderCallSuccess {
    pub results: Vec<ProviderNewsItem>,
    pub latency_ms: u64,
}

pub fn execute_brave_news_search(
    endpoint: &str,
    api_key: &str,
    query: &str,
    max_results: usize,
    timeout_ms: u64,
    user_agent: &str,
    proxy_config: &crate::web_search_plan::proxy::proxy_config::ProxyConfig,
) -> Result<ProviderCallSuccess, NewsProviderError> {
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
            return Err(NewsProviderError {
                provider_id: NewsProviderId::BraveNewsSearch,
                kind: NewsProviderErrorKind::ProxyMisconfigured,
                status_code: None,
                message: check.details,
                latency_ms: start.elapsed().as_millis() as u64,
            });
        }
    }

    if let Some(proxy_raw) = select_proxy_url(endpoint, proxy_config) {
        let _ = redact_proxy_url(proxy_raw).map_err(|_| NewsProviderError {
            provider_id: NewsProviderId::BraveNewsSearch,
            kind: NewsProviderErrorKind::ProxyMisconfigured,
            status_code: None,
            message: "proxy URL redaction failed".to_string(),
            latency_ms: start.elapsed().as_millis() as u64,
        })?;

        let proxy = ureq::Proxy::new(proxy_raw).map_err(|_| NewsProviderError {
            provider_id: NewsProviderId::BraveNewsSearch,
            kind: NewsProviderErrorKind::ProxyMisconfigured,
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
        .map_err(|err| map_ureq_error(err, start.elapsed().as_millis() as u64))?;

    let body: Value =
        serde_json::from_reader(response.into_reader()).map_err(|_| NewsProviderError {
            provider_id: NewsProviderId::BraveNewsSearch,
            kind: NewsProviderErrorKind::ParseFailed,
            status_code: None,
            message: "brave news JSON parse failed".to_string(),
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
            .filter(|value| !value.is_empty())
        else {
            continue;
        };

        let canonical = canonicalize_url(url).map_err(|_| NewsProviderError {
            provider_id: NewsProviderId::BraveNewsSearch,
            kind: NewsProviderErrorKind::ParseFailed,
            status_code: None,
            message: format!("invalid brave news URL {}", url),
            latency_ms: start.elapsed().as_millis() as u64,
        })?;

        let title = item
            .get("title")
            .or_else(|| item.get("name"))
            .and_then(Value::as_str)
            .map(normalize_text_value)
            .filter(|v| !v.is_empty())
            .unwrap_or_else(|| "News Result".to_string());

        let snippet = item
            .get("description")
            .or_else(|| item.get("snippet"))
            .or_else(|| item.get("content"))
            .and_then(Value::as_str)
            .map(normalize_text_value)
            .filter(|v| !v.is_empty())
            .unwrap_or_else(|| "No snippet available.".to_string());

        let published_raw = item
            .get("published")
            .or_else(|| item.get("page_age"))
            .or_else(|| item.get("age"))
            .or_else(|| item.get("date"))
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|v| !v.is_empty())
            .map(ToString::to_string);

        let trust_tier = item
            .get("trust_tier")
            .or_else(|| item.get("source_rank"))
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|v| !v.is_empty())
            .map(ToString::to_string);

        results.push(ProviderNewsItem {
            title,
            url: url.to_string(),
            snippet,
            canonical_url: canonical.canonical_url,
            published_raw,
            provider_id: NewsProviderId::BraveNewsSearch,
            provider_rank: results.len() + 1,
            trust_tier,
            domain: extract_domain(url).unwrap_or_default(),
        });
    }

    if results.is_empty() {
        return Err(NewsProviderError {
            provider_id: NewsProviderId::BraveNewsSearch,
            kind: NewsProviderErrorKind::EmptyResults,
            status_code: None,
            message: "brave news returned no usable results".to_string(),
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
    if let Some(results) = body.pointer("/results").and_then(Value::as_array) {
        out.extend(results.iter());
    }
    if let Some(results) = body.pointer("/news/results").and_then(Value::as_array) {
        out.extend(results.iter());
    }
    out
}

fn select_proxy_url<'a>(
    endpoint: &str,
    proxy_config: &'a crate::web_search_plan::proxy::proxy_config::ProxyConfig,
) -> Option<&'a str> {
    let is_https = endpoint.trim().to_ascii_lowercase().starts_with("https://");
    if is_https {
        proxy_config.https_proxy_url.as_deref()
    } else {
        proxy_config.http_proxy_url.as_deref()
    }
}

fn map_ureq_error(err: ureq::Error, latency_ms: u64) -> NewsProviderError {
    match err {
        ureq::Error::Status(status, _) => NewsProviderError {
            provider_id: NewsProviderId::BraveNewsSearch,
            kind: NewsProviderErrorKind::HttpNon200,
            status_code: Some(status as u16),
            message: format!("brave news HTTP status {}", status),
            latency_ms,
        },
        ureq::Error::Transport(transport) => {
            let combined = format!("{:?} {}", transport.kind(), transport).to_ascii_lowercase();
            let kind = if combined.contains("timeout") {
                NewsProviderErrorKind::TimeoutExceeded
            } else if combined.contains("tls") || combined.contains("ssl") {
                NewsProviderErrorKind::TlsFailed
            } else if combined.contains("dns") {
                NewsProviderErrorKind::DnsFailed
            } else if combined.contains("connect") || combined.contains("connection") {
                NewsProviderErrorKind::ConnectFailed
            } else {
                NewsProviderErrorKind::TransportFailed
            };

            NewsProviderError {
                provider_id: NewsProviderId::BraveNewsSearch,
                kind,
                status_code: None,
                message: "brave news transport failure".to_string(),
                latency_ms,
            }
        }
    }
}
