#![forbid(unsafe_code)]

pub mod finance;
pub mod flights;
pub mod generic_json;
pub mod weather;

use crate::web_search_plan::perf_cost::tiers::ImportanceTier;
use crate::web_search_plan::perf_cost::timeouts::clamp_provider_timeout;
use crate::web_search_plan::proxy::ProxyMode;
use crate::web_search_plan::realtime::domains::RealtimeDomain;
use crate::web_search_plan::realtime::{
    ParsedRealtimeToolRequest, RealtimeError, RealtimeErrorKind, RealtimeRuntimeConfig,
};
use selene_kernel_contracts::provider_secrets::ProviderSecretId;
use serde_json::Value;
use std::io::Read;
use std::time::Duration;

#[derive(Debug, Clone, PartialEq)]
pub struct RealtimeAdapterOutput {
    pub provider_id: String,
    pub endpoint_url: String,
    pub title: String,
    pub trust_tier: String,
    pub retrieved_at_ms: i64,
    pub latency_ms: u64,
    pub payload: Value,
}

pub fn run_adapter(
    request: &ParsedRealtimeToolRequest,
    config: &RealtimeRuntimeConfig,
) -> Result<RealtimeAdapterOutput, RealtimeError> {
    match request.domain {
        RealtimeDomain::Weather => weather::execute(request, config),
        RealtimeDomain::Finance => finance::execute(request, config),
        RealtimeDomain::Flights => flights::execute(request, config),
        RealtimeDomain::GenericRealTime => generic_json::execute(request, config),
    }
}

pub fn fetch_json_with_caps(
    adapter_id: &str,
    request: &ParsedRealtimeToolRequest,
    config: &RealtimeRuntimeConfig,
    url: &str,
    headers: &[(&str, &str)],
) -> Result<(Value, u64), RealtimeError> {
    let tier = ImportanceTier::parse_or_default(request.importance_tier.as_str());
    let timeout_ms = clamp_provider_timeout(config.timeout_ms, tier);

    let mut builder = ureq::AgentBuilder::new()
        .timeout_connect(Duration::from_millis(timeout_ms))
        .timeout_read(Duration::from_millis(timeout_ms))
        .timeout_write(Duration::from_millis(timeout_ms))
        .user_agent(config.user_agent.as_str())
        .try_proxy_from_env(false)
        .redirects(0);

    if let Some(proxy_url) = proxy_for_url(request, url) {
        let proxy = ureq::Proxy::new(proxy_url).map_err(|_| {
            RealtimeError::new(
                adapter_id,
                RealtimeErrorKind::PolicyViolation,
                None,
                "proxy configuration invalid",
                0,
            )
        })?;
        builder = builder.proxy(proxy);
    }

    let agent = builder.build();
    let mut req = agent
        .get(url)
        .set("Accept", "application/json")
        .set("Cache-Control", "no-cache")
        .set("Pragma", "no-cache")
        .timeout(Duration::from_millis(timeout_ms));

    for (key, value) in headers {
        req = req.set(key, value);
    }

    let started = std::time::Instant::now();
    let response = match req.call() {
        Ok(resp) => resp,
        Err(ureq::Error::Status(code, _)) => {
            return Err(RealtimeError::new(
                adapter_id,
                RealtimeErrorKind::ProviderUpstreamFailed,
                Some(code),
                format!("http status {}", code),
                started.elapsed().as_millis() as u64,
            ));
        }
        Err(ureq::Error::Transport(error)) => {
            let message = format!("{:?} {}", error.kind(), error).to_ascii_lowercase();
            let kind = if message.contains("timeout") {
                RealtimeErrorKind::TimeoutExceeded
            } else {
                RealtimeErrorKind::ProviderUpstreamFailed
            };
            return Err(RealtimeError::new(
                adapter_id,
                kind,
                None,
                "transport failure during realtime fetch",
                started.elapsed().as_millis() as u64,
            ));
        }
    };

    let mut body = Vec::new();
    let mut reader = response
        .into_reader()
        .take(config.max_response_bytes as u64 + 1);
    reader.read_to_end(&mut body).map_err(|error| {
        RealtimeError::new(
            adapter_id,
            RealtimeErrorKind::ProviderUpstreamFailed,
            None,
            format!("failed reading realtime response: {}", error),
            started.elapsed().as_millis() as u64,
        )
    })?;

    if body.len() > config.max_response_bytes {
        return Err(RealtimeError::new(
            adapter_id,
            RealtimeErrorKind::PolicyViolation,
            None,
            "realtime response exceeded max_response_bytes",
            started.elapsed().as_millis() as u64,
        ));
    }

    let parsed = serde_json::from_slice::<Value>(&body).map_err(|error| {
        RealtimeError::new(
            adapter_id,
            RealtimeErrorKind::ProviderUpstreamFailed,
            None,
            format!("realtime response JSON parse failed: {}", error),
            started.elapsed().as_millis() as u64,
        )
    })?;
    Ok((parsed, started.elapsed().as_millis() as u64))
}

pub fn parse_retrieved_at_ms(adapter_id: &str, payload: &Value) -> Result<i64, RealtimeError> {
    payload
        .get("retrieved_at_ms")
        .and_then(Value::as_i64)
        .filter(|value| *value > 0)
        .ok_or_else(|| {
            RealtimeError::new(
                adapter_id,
                RealtimeErrorKind::PolicyViolation,
                None,
                "realtime payload missing retrieved_at_ms",
                0,
            )
        })
}

pub fn parse_trust_tier(payload: &Value) -> String {
    payload
        .get("trust_tier")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("medium")
        .to_ascii_lowercase()
}

pub fn resolve_api_key(
    env_var: &str,
    env_override: &Option<String>,
    vault_secret_override: &Option<String>,
    default_secret: Option<ProviderSecretId>,
) -> Option<String> {
    if let Some(value) = env_override.as_ref() {
        let trimmed = value.trim();
        return if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        };
    }
    if let Ok(value) = std::env::var(env_var) {
        let trimmed = value.trim().to_string();
        if !trimmed.is_empty() {
            return Some(trimmed);
        }
    }

    let secret_id = vault_secret_override
        .as_deref()
        .and_then(ProviderSecretId::parse)
        .or(default_secret)?;
    resolve_secret_from_vault(secret_id)
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

fn proxy_for_url<'a>(request: &'a ParsedRealtimeToolRequest, url: &str) -> Option<&'a str> {
    let is_https = url.to_ascii_lowercase().starts_with("https://");
    match request.proxy_config.mode {
        ProxyMode::Off => None,
        ProxyMode::Env | ProxyMode::Explicit => {
            if is_https {
                request.proxy_config.https_proxy_url.as_deref()
            } else {
                request.proxy_config.http_proxy_url.as_deref()
            }
        }
    }
}
