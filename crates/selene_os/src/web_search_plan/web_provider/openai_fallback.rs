#![forbid(unsafe_code)]

use crate::web_search_plan::proxy::proxy_redaction::redact_proxy_url;
use crate::web_search_plan::proxy::proxy_self_check::run_startup_self_check;
use crate::web_search_plan::proxy::ProxyErrorKind;
use crate::web_search_plan::url::canonical::canonicalize_url;
use crate::web_search_plan::web_provider::{
    normalize_text_value, NormalizedSearchResult, ProviderCallSuccess, ProviderError,
    ProviderErrorKind, ProviderId,
};
use serde_json::{json, Value};
use std::collections::BTreeSet;
use std::time::{Duration, Instant};

pub fn execute_openai_web_search(
    endpoint: &str,
    api_key: &str,
    model: &str,
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
                provider_id: ProviderId::OpenAiWebSearch,
                kind: ProviderErrorKind::ProxyMisconfigured,
                status_code: None,
                message: check.details,
                latency_ms: start.elapsed().as_millis() as u64,
            });
        }
    }

    if let Some(proxy_raw) = select_proxy_url(endpoint, proxy_config) {
        let _ = redact_proxy_url(proxy_raw).map_err(|_| ProviderError {
            provider_id: ProviderId::OpenAiWebSearch,
            kind: ProviderErrorKind::ProxyMisconfigured,
            status_code: None,
            message: "proxy URL redaction failed".to_string(),
            latency_ms: start.elapsed().as_millis() as u64,
        })?;
        let proxy = ureq::Proxy::new(proxy_raw).map_err(|_| ProviderError {
            provider_id: ProviderId::OpenAiWebSearch,
            kind: ProviderErrorKind::ProxyMisconfigured,
            status_code: None,
            message: "invalid proxy URL".to_string(),
            latency_ms: start.elapsed().as_millis() as u64,
        })?;
        builder = builder.proxy(proxy);
    }

    let prompt = format!(
        "Return strict JSON with key results (array of title,url,snippet) for query: {}. Use exactly {} results maximum and include only URLs that are explicitly cited by web search.",
        query, max_results
    );

    let payload = json!({
        "model": model,
        "input": prompt,
        "temperature": 0,
        "max_output_tokens": 900,
        "tools": [{"type": "web_search"}]
    });

    let agent = builder.build();
    let response = agent
        .post(endpoint)
        .set("Accept", "application/json")
        .set("Content-Type", "application/json")
        .set("Authorization", &format!("Bearer {}", api_key))
        .send_json(payload)
        .map_err(|err| {
            map_ureq_error(
                ProviderId::OpenAiWebSearch,
                err,
                start.elapsed().as_millis() as u64,
            )
        })?;

    let body: Value =
        serde_json::from_reader(response.into_reader()).map_err(|_| ProviderError {
            provider_id: ProviderId::OpenAiWebSearch,
            kind: ProviderErrorKind::ParseFailed,
            status_code: None,
            message: "openai JSON parse failed".to_string(),
            latency_ms: start.elapsed().as_millis() as u64,
        })?;

    openai_web_search_from_body(&body, max_results, start.elapsed().as_millis() as u64)
}

pub fn openai_web_search_from_body(
    body: &Value,
    max_results: usize,
    latency_ms: u64,
) -> Result<ProviderCallSuccess, ProviderError> {
    let citation_urls = collect_citation_urls(body);
    if citation_urls.is_empty() {
        return Err(ProviderError {
            provider_id: ProviderId::OpenAiWebSearch,
            kind: ProviderErrorKind::ParseFailed,
            status_code: None,
            message: "openai response did not include explicit citation URLs".to_string(),
            latency_ms,
        });
    }

    let raw_items = extract_result_items(body, max_results).ok_or_else(|| ProviderError {
        provider_id: ProviderId::OpenAiWebSearch,
        kind: ProviderErrorKind::ParseFailed,
        status_code: None,
        message: "openai response did not contain parseable results".to_string(),
        latency_ms,
    })?;

    let mut results = Vec::new();
    for raw in raw_items {
        let canonical = canonicalize_url(raw.url.as_str()).map_err(|_| ProviderError {
            provider_id: ProviderId::OpenAiWebSearch,
            kind: ProviderErrorKind::ParseFailed,
            status_code: None,
            message: "openai result URL is not canonicalizable".to_string(),
            latency_ms,
        })?;

        if !citation_urls.contains(canonical.canonical_url.as_str()) {
            return Err(ProviderError {
                provider_id: ProviderId::OpenAiWebSearch,
                kind: ProviderErrorKind::ParseFailed,
                status_code: None,
                message: "openai result URL is missing explicit citation coverage".to_string(),
                latency_ms,
            });
        }

        results.push(NormalizedSearchResult {
            title: raw.title,
            url: raw.url.clone(),
            snippet: raw.snippet,
            canonical_url: canonical.canonical_url,
            citation_url: raw.url,
            provider_id: ProviderId::OpenAiWebSearch,
            provider_rank: results.len() + 1,
        });
    }

    if results.is_empty() {
        return Err(ProviderError {
            provider_id: ProviderId::OpenAiWebSearch,
            kind: ProviderErrorKind::EmptyResults,
            status_code: None,
            message: "openai returned zero usable results".to_string(),
            latency_ms,
        });
    }

    Ok(ProviderCallSuccess {
        results,
        latency_ms,
    })
}

#[derive(Debug, Clone)]
struct RawResultItem {
    title: String,
    url: String,
    snippet: String,
}

fn extract_result_items(root: &Value, max_results: usize) -> Option<Vec<RawResultItem>> {
    if let Some(items) = root.get("results").and_then(Value::as_array) {
        return Some(extract_items_from_array(items, max_results));
    }

    let output_text = root
        .get("output_text")
        .and_then(Value::as_str)
        .or_else(|| {
            root.pointer("/output/0/content/0/text")
                .and_then(Value::as_str)
        })?;

    let start = output_text.find('{')?;
    let end = output_text.rfind('}')?;
    let json_candidate = &output_text[start..=end];
    let parsed = serde_json::from_str::<Value>(json_candidate).ok()?;
    let items = parsed.get("results")?.as_array()?;
    Some(extract_items_from_array(items, max_results))
}

fn extract_items_from_array(items: &[Value], max_results: usize) -> Vec<RawResultItem> {
    let mut out = Vec::new();
    for item in items {
        if out.len() >= max_results {
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

        let title = item
            .get("title")
            .or_else(|| item.get("name"))
            .and_then(Value::as_str)
            .map(normalize_text_value)
            .filter(|v| !v.is_empty())
            .unwrap_or_else(|| "Result".to_string());

        let snippet = item
            .get("snippet")
            .or_else(|| item.get("description"))
            .and_then(Value::as_str)
            .map(normalize_text_value)
            .filter(|v| !v.is_empty())
            .unwrap_or_else(|| "No snippet available.".to_string());

        out.push(RawResultItem {
            title,
            url: url.to_string(),
            snippet,
        });
    }
    out
}

fn collect_citation_urls(root: &Value) -> BTreeSet<String> {
    let mut out = BTreeSet::new();

    if let Some(citations) = root.get("citations").and_then(Value::as_array) {
        for citation in citations {
            if let Some(url) = citation.get("url").and_then(Value::as_str) {
                if let Ok(canonical) = canonicalize_url(url) {
                    out.insert(canonical.canonical_url);
                }
            }
        }
    }

    if let Some(output) = root.get("output").and_then(Value::as_array) {
        for item in output {
            if let Some(content) = item.get("content").and_then(Value::as_array) {
                for block in content {
                    if let Some(annotations) = block.get("annotations").and_then(Value::as_array) {
                        for annotation in annotations {
                            let annotation_type = annotation
                                .get("type")
                                .and_then(Value::as_str)
                                .unwrap_or_default()
                                .to_ascii_lowercase();
                            if !annotation_type.contains("citation") {
                                continue;
                            }
                            if let Some(url) = annotation.get("url").and_then(Value::as_str) {
                                if let Ok(canonical) = canonicalize_url(url) {
                                    out.insert(canonical.canonical_url);
                                }
                            }
                        }
                    }
                }
            }
        }
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
