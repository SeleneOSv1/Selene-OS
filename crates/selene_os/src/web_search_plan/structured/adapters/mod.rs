#![forbid(unsafe_code)]

pub mod academic;
pub mod company_registry;
pub mod filings;
pub mod generic_http_json;
pub mod gov_dataset;
pub mod patents;
pub mod pricing_products;

use crate::web_search_plan::perf_cost::tiers::ImportanceTier;
use crate::web_search_plan::perf_cost::timeouts::clamp_provider_timeout;
use crate::web_search_plan::proxy::ProxyMode;
use crate::web_search_plan::structured::registry::AdapterSelection;
use crate::web_search_plan::structured::types::{
    StructuredAdapterOutput, StructuredAdapterRequest, StructuredConnectorError,
    StructuredErrorKind, StructuredRuntimeConfig,
};
use serde_json::Value;
use std::io::Read;
use std::time::Duration;

pub fn run_adapter(
    selection: AdapterSelection,
    request: &StructuredAdapterRequest,
    config: &StructuredRuntimeConfig,
) -> Result<StructuredAdapterOutput, StructuredConnectorError> {
    match selection {
        AdapterSelection::GenericHttpJson => generic_http_json::execute(request, config),
        AdapterSelection::GovDataset => gov_dataset::execute(request, config),
        AdapterSelection::CompanyRegistry => company_registry::execute(request, config),
        AdapterSelection::Filings => filings::execute(request, config),
        AdapterSelection::Patents => patents::execute(request, config),
        AdapterSelection::Academic => academic::execute(request, config),
        AdapterSelection::PricingProducts => pricing_products::execute(request, config),
    }
}

pub fn fetch_json_with_caps(
    adapter_id: &str,
    request: &StructuredAdapterRequest,
    config: &StructuredRuntimeConfig,
    url: &str,
    headers: &[(&str, &str)],
) -> Result<(Value, u64), StructuredConnectorError> {
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
            StructuredConnectorError::new(
                adapter_id,
                StructuredErrorKind::PolicyViolation,
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
            return Err(StructuredConnectorError::new(
                adapter_id,
                StructuredErrorKind::ProviderUpstreamFailed,
                Some(code),
                format!("http status {}", code),
                started.elapsed().as_millis() as u64,
            ));
        }
        Err(ureq::Error::Transport(error)) => {
            let message = format!("{:?} {}", error.kind(), error).to_ascii_lowercase();
            let kind = if message.contains("timeout") {
                StructuredErrorKind::TimeoutExceeded
            } else {
                StructuredErrorKind::ProviderUpstreamFailed
            };
            return Err(StructuredConnectorError::new(
                adapter_id,
                kind,
                None,
                "transport failure during structured fetch",
                started.elapsed().as_millis() as u64,
            ));
        }
    };

    let mut body = Vec::new();
    let mut reader = response
        .into_reader()
        .take(config.max_response_bytes as u64 + 1);
    reader.read_to_end(&mut body).map_err(|error| {
        StructuredConnectorError::new(
            adapter_id,
            StructuredErrorKind::ProviderUpstreamFailed,
            None,
            format!("failed to read response body: {}", error),
            started.elapsed().as_millis() as u64,
        )
    })?;

    if body.len() > config.max_response_bytes {
        return Err(StructuredConnectorError::new(
            adapter_id,
            StructuredErrorKind::PolicyViolation,
            None,
            "structured response exceeded max_response_bytes",
            started.elapsed().as_millis() as u64,
        ));
    }

    let parsed = serde_json::from_slice::<Value>(&body).map_err(|error| {
        StructuredConnectorError::new(
            adapter_id,
            StructuredErrorKind::ProviderUpstreamFailed,
            None,
            format!("response JSON parse failed: {}", error),
            started.elapsed().as_millis() as u64,
        )
    })?;

    Ok((parsed, started.elapsed().as_millis() as u64))
}

fn proxy_for_url<'a>(request: &'a StructuredAdapterRequest, url: &str) -> Option<&'a str> {
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
