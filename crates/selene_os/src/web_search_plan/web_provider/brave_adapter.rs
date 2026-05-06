#![forbid(unsafe_code)]

use crate::web_search_plan::proxy::proxy_redaction::parse_proxy_endpoint;
use crate::web_search_plan::proxy::proxy_redaction::redact_proxy_url;
use crate::web_search_plan::proxy::proxy_self_check::run_startup_self_check;
use crate::web_search_plan::proxy::ProxyErrorKind;
use crate::web_search_plan::proxy::ProxyMode;
use crate::web_search_plan::url::canonical::canonicalize_url;
use crate::web_search_plan::web_provider::{
    normalize_text_value, NormalizedSearchResult, ProviderCallSuccess, ProviderError,
    ProviderErrorKind, ProviderId,
};
use serde_json::Value;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::Arc;
use std::time::{Duration, Instant};
use url::Url;

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
    if should_use_manual_http_connect(endpoint, proxy_config) {
        let response = execute_brave_manual_http_connect(
            endpoint,
            Some(api_key),
            query,
            max_results,
            timeout_ms,
            user_agent,
            proxy_config,
            start,
        )?;
        if response.status != 200 {
            return Err(ProviderError {
                provider_id: ProviderId::BraveWebSearch,
                kind: ProviderErrorKind::HttpNon200,
                status_code: Some(response.status),
                message: format!("provider returned HTTP status {}", response.status),
                latency_ms: start.elapsed().as_millis() as u64,
            });
        }
        let body: Value = serde_json::from_slice(&response.body).map_err(|_| ProviderError {
            provider_id: ProviderId::BraveWebSearch,
            kind: ProviderErrorKind::ParseFailed,
            status_code: None,
            message: "brave JSON parse failed".to_string(),
            latency_ms: start.elapsed().as_millis() as u64,
        })?;
        return brave_web_search_from_body(&body, max_results, start.elapsed().as_millis() as u64);
    }

    let agent = build_brave_agent(endpoint, timeout_ms, user_agent, proxy_config, start)?;
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

    brave_web_search_from_body(&body, max_results, start.elapsed().as_millis() as u64)
}

pub fn probe_brave_web_search_transport_without_secret(
    endpoint: &str,
    query: &str,
    timeout_ms: u64,
    user_agent: &str,
    proxy_config: &crate::web_search_plan::proxy::proxy_config::ProxyConfig,
) -> Result<u16, ProviderError> {
    let start = Instant::now();
    if should_use_manual_http_connect(endpoint, proxy_config) {
        return execute_brave_manual_http_connect(
            endpoint,
            None,
            query,
            2,
            timeout_ms,
            user_agent,
            proxy_config,
            start,
        )
        .map(|response| response.status);
    }

    let agent = build_brave_agent(endpoint, timeout_ms, user_agent, proxy_config, start)?;
    match agent
        .get(endpoint)
        .set("Accept", "application/json")
        .query("q", query)
        .query("count", "2")
        .call()
    {
        Ok(response) => Ok(response.status()),
        Err(ureq::Error::Status(status, _)) => Ok(status as u16),
        Err(err) => Err(map_ureq_error(
            ProviderId::BraveWebSearch,
            err,
            start.elapsed().as_millis() as u64,
        )),
    }
}

fn build_brave_agent(
    endpoint: &str,
    timeout_ms: u64,
    user_agent: &str,
    proxy_config: &crate::web_search_plan::proxy::proxy_config::ProxyConfig,
    start: Instant,
) -> Result<ureq::Agent, ProviderError> {
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

    Ok(builder.build())
}

pub fn brave_web_search_from_body(
    body: &Value,
    max_results: usize,
    latency_ms: u64,
) -> Result<ProviderCallSuccess, ProviderError> {
    let mut results = Vec::new();
    for item in candidate_results(body) {
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
            latency_ms,
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
            latency_ms,
        });
    }

    Ok(ProviderCallSuccess {
        results,
        latency_ms,
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

#[derive(Debug)]
struct ManualHttpResponse {
    status: u16,
    body: Vec<u8>,
}

fn should_use_manual_http_connect(
    endpoint: &str,
    config: &crate::web_search_plan::proxy::proxy_config::ProxyConfig,
) -> bool {
    endpoint.trim().to_ascii_lowercase().starts_with("https://")
        && config.mode == ProxyMode::Explicit
        && config
            .https_proxy_url
            .as_deref()
            .map(|url| url.trim().to_ascii_lowercase().starts_with("http://"))
            .unwrap_or(false)
}

fn execute_brave_manual_http_connect(
    endpoint: &str,
    api_key: Option<&str>,
    query: &str,
    max_results: usize,
    timeout_ms: u64,
    user_agent: &str,
    proxy_config: &crate::web_search_plan::proxy::proxy_config::ProxyConfig,
    start: Instant,
) -> Result<ManualHttpResponse, ProviderError> {
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

    let url = build_brave_request_url(endpoint, query, max_results, start)?;
    let host = url.host_str().ok_or_else(|| ProviderError {
        provider_id: ProviderId::BraveWebSearch,
        kind: ProviderErrorKind::ProxyMisconfigured,
        status_code: None,
        message: "brave endpoint host missing".to_string(),
        latency_ms: start.elapsed().as_millis() as u64,
    })?;
    let port = url.port_or_known_default().unwrap_or(443);
    let proxy_raw = proxy_config
        .https_proxy_url
        .as_deref()
        .ok_or_else(|| ProviderError {
            provider_id: ProviderId::BraveWebSearch,
            kind: ProviderErrorKind::ProxyMisconfigured,
            status_code: None,
            message: "missing HTTPS proxy URL".to_string(),
            latency_ms: start.elapsed().as_millis() as u64,
        })?;
    let proxy = parse_proxy_endpoint(proxy_raw).map_err(|_| ProviderError {
        provider_id: ProviderId::BraveWebSearch,
        kind: ProviderErrorKind::ProxyMisconfigured,
        status_code: None,
        message: "invalid proxy URL".to_string(),
        latency_ms: start.elapsed().as_millis() as u64,
    })?;

    let timeout = Duration::from_millis(timeout_ms);
    let mut stream = TcpStream::connect((proxy.host.as_str(), proxy.port)).map_err(|err| {
        map_io_provider_error(
            ProviderErrorKind::ConnectFailed,
            format!("proxy TCP connect failed: {}", err.kind()),
            start,
        )
    })?;
    stream.set_read_timeout(Some(timeout)).map_err(|err| {
        map_io_provider_error(
            ProviderErrorKind::TransportFailed,
            format!("proxy read timeout setup failed: {}", err.kind()),
            start,
        )
    })?;
    stream.set_write_timeout(Some(timeout)).map_err(|err| {
        map_io_provider_error(
            ProviderErrorKind::TransportFailed,
            format!("proxy write timeout setup failed: {}", err.kind()),
            start,
        )
    })?;

    let connect = format!(
        "CONNECT {host}:{port} HTTP/1.1\r\nHost: {host}:{port}\r\nUser-Agent: {user_agent}\r\nProxy-Connection: close\r\n\r\n"
    );
    stream.write_all(connect.as_bytes()).map_err(|err| {
        map_io_provider_error(
            ProviderErrorKind::ConnectFailed,
            format!("proxy CONNECT write failed: {}", err.kind()),
            start,
        )
    })?;
    stream.flush().map_err(|err| {
        map_io_provider_error(
            ProviderErrorKind::ConnectFailed,
            format!("proxy CONNECT flush failed: {}", err.kind()),
            start,
        )
    })?;

    let connect_head = read_http_head(&mut stream, start)?;
    let connect_status = parse_http_status(&connect_head).ok_or_else(|| ProviderError {
        provider_id: ProviderId::BraveWebSearch,
        kind: ProviderErrorKind::ProxyMisconfigured,
        status_code: None,
        message: "proxy CONNECT response status parse failed".to_string(),
        latency_ms: start.elapsed().as_millis() as u64,
    })?;
    if connect_status != 200 {
        return Err(ProviderError {
            provider_id: ProviderId::BraveWebSearch,
            kind: ProviderErrorKind::ProxyMisconfigured,
            status_code: Some(connect_status),
            message: format!("proxy CONNECT returned HTTP status {}", connect_status),
            latency_ms: start.elapsed().as_millis() as u64,
        });
    }

    let _ = rustls::crypto::ring::default_provider().install_default();
    let roots = rustls::RootCertStore {
        roots: webpki_roots::TLS_SERVER_ROOTS.to_vec(),
    };
    let mut tls_config = rustls::ClientConfig::builder()
        .with_root_certificates(roots)
        .with_no_client_auth();
    tls_config.alpn_protocols.push(b"http/1.1".to_vec());
    let server_name = host.to_string().try_into().map_err(|_| ProviderError {
        provider_id: ProviderId::BraveWebSearch,
        kind: ProviderErrorKind::TlsFailed,
        status_code: None,
        message: "brave TLS server name parse failed".to_string(),
        latency_ms: start.elapsed().as_millis() as u64,
    })?;
    let mut tls_conn =
        rustls::ClientConnection::new(Arc::new(tls_config), server_name).map_err(|_| {
            ProviderError {
                provider_id: ProviderId::BraveWebSearch,
                kind: ProviderErrorKind::TlsFailed,
                status_code: None,
                message: "rustls client connection setup failed".to_string(),
                latency_ms: start.elapsed().as_millis() as u64,
            }
        })?;
    let mut tls = rustls::Stream::new(&mut tls_conn, &mut stream);

    let target = request_target(&url);
    let mut request = format!(
        "GET {target} HTTP/1.1\r\nHost: {host}\r\nUser-Agent: {user_agent}\r\nAccept: application/json\r\nConnection: close\r\n"
    );
    if let Some(api_key) = api_key {
        request.push_str("X-Subscription-Token: ");
        request.push_str(api_key);
        request.push_str("\r\n");
    }
    request.push_str("\r\n");

    tls.write_all(request.as_bytes()).map_err(|err| {
        map_io_provider_error(
            ProviderErrorKind::TransportFailed,
            format!("brave request write failed: {}", err.kind()),
            start,
        )
    })?;
    tls.flush().map_err(|err| {
        map_io_provider_error(
            ProviderErrorKind::TransportFailed,
            format!("brave request flush failed: {}", err.kind()),
            start,
        )
    })?;

    let mut response = Vec::new();
    tls.read_to_end(&mut response).map_err(|err| {
        map_io_provider_error(
            ProviderErrorKind::TransportFailed,
            format!("brave response read failed: {}", err.kind()),
            start,
        )
    })?;
    split_http_response(&response, start)
}

fn build_brave_request_url(
    endpoint: &str,
    query: &str,
    max_results: usize,
    start: Instant,
) -> Result<Url, ProviderError> {
    let mut url = Url::parse(endpoint).map_err(|_| ProviderError {
        provider_id: ProviderId::BraveWebSearch,
        kind: ProviderErrorKind::ProxyMisconfigured,
        status_code: None,
        message: "brave endpoint URL parse failed".to_string(),
        latency_ms: start.elapsed().as_millis() as u64,
    })?;
    url.query_pairs_mut()
        .append_pair("q", query)
        .append_pair("count", &max_results.to_string());
    Ok(url)
}

fn request_target(url: &Url) -> String {
    match url.query() {
        Some(query) => format!("{}?{}", url.path(), query),
        None => url.path().to_string(),
    }
}

fn read_http_head(stream: &mut TcpStream, start: Instant) -> Result<Vec<u8>, ProviderError> {
    let mut head = Vec::new();
    let mut byte = [0_u8; 1];
    while !head.ends_with(b"\r\n\r\n") {
        let count = stream.read(&mut byte).map_err(|err| {
            map_io_provider_error(
                ProviderErrorKind::ConnectFailed,
                format!("proxy CONNECT response read failed: {}", err.kind()),
                start,
            )
        })?;
        if count == 0 {
            break;
        }
        head.push(byte[0]);
        if head.len() > 8192 {
            return Err(ProviderError {
                provider_id: ProviderId::BraveWebSearch,
                kind: ProviderErrorKind::ProxyMisconfigured,
                status_code: None,
                message: "proxy CONNECT response head too large".to_string(),
                latency_ms: start.elapsed().as_millis() as u64,
            });
        }
    }
    Ok(head)
}

fn split_http_response(
    response: &[u8],
    start: Instant,
) -> Result<ManualHttpResponse, ProviderError> {
    let Some(head_end) = response.windows(4).position(|window| window == b"\r\n\r\n") else {
        return Err(ProviderError {
            provider_id: ProviderId::BraveWebSearch,
            kind: ProviderErrorKind::TransportFailed,
            status_code: None,
            message: "brave response head parse failed".to_string(),
            latency_ms: start.elapsed().as_millis() as u64,
        });
    };
    let head = &response[..head_end + 4];
    let status = parse_http_status(head).ok_or_else(|| ProviderError {
        provider_id: ProviderId::BraveWebSearch,
        kind: ProviderErrorKind::TransportFailed,
        status_code: None,
        message: "brave response status parse failed".to_string(),
        latency_ms: start.elapsed().as_millis() as u64,
    })?;
    Ok(ManualHttpResponse {
        status,
        body: response[head_end + 4..].to_vec(),
    })
}

fn parse_http_status(head: &[u8]) -> Option<u16> {
    let line_end = head.windows(2).position(|window| window == b"\r\n")?;
    let line = std::str::from_utf8(&head[..line_end]).ok()?;
    let mut parts = line.split_whitespace();
    let _http = parts.next()?;
    parts.next()?.parse::<u16>().ok()
}

fn map_io_provider_error(
    kind: ProviderErrorKind,
    message: String,
    start: Instant,
) -> ProviderError {
    ProviderError {
        provider_id: ProviderId::BraveWebSearch,
        kind,
        status_code: None,
        message,
        latency_ms: start.elapsed().as_millis() as u64,
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
            let detail = sanitized_transport_detail(&transport);
            let combined = detail.to_ascii_lowercase();
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
                message: format!("transport failure during provider call: {}", detail),
                latency_ms,
            }
        }
    }
}

fn sanitized_transport_detail(transport: &ureq::Transport) -> String {
    let raw = format!("{:?}: {}", transport.kind(), transport);
    raw.chars().take(240).collect()
}
