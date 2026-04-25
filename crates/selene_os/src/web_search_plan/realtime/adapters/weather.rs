#![forbid(unsafe_code)]

use crate::web_search_plan::realtime::adapters::{
    fetch_json_with_caps, resolve_api_key, RealtimeAdapterOutput,
};
use crate::web_search_plan::realtime::{
    ParsedRealtimeToolRequest, RealtimeError, RealtimeErrorKind, RealtimeRuntimeConfig,
};
use selene_kernel_contracts::provider_secrets::ProviderSecretId;
use serde_json::{json, Value};
use url::Url;

const TOMORROW_IO_PROVIDER_ID: &str = "TomorrowIoWeather";
const WEATHER_API_PROVIDER_ID: &str = "WeatherApiWeather";

pub fn execute(
    request: &ParsedRealtimeToolRequest,
    config: &RealtimeRuntimeConfig,
) -> Result<RealtimeAdapterOutput, RealtimeError> {
    let primary = execute_tomorrow_io(request, config);
    match primary {
        Ok(output) => return Ok(output),
        Err(primary_error) => match primary_error.kind {
            RealtimeErrorKind::ProviderUnconfigured
            | RealtimeErrorKind::ProviderUpstreamFailed
            | RealtimeErrorKind::TimeoutExceeded => {
                return execute_weather_api(request, config).map_err(|secondary_error| {
                    if secondary_error.kind == RealtimeErrorKind::ProviderUnconfigured
                        && primary_error.kind != RealtimeErrorKind::ProviderUnconfigured
                    {
                        primary_error
                    } else {
                        secondary_error
                    }
                });
            }
            _ => return Err(primary_error),
        },
    }
}

fn execute_tomorrow_io(
    request: &ParsedRealtimeToolRequest,
    config: &RealtimeRuntimeConfig,
) -> Result<RealtimeAdapterOutput, RealtimeError> {
    let api_key = resolve_api_key(
        "SELENE_REALTIME_TOMORROW_IO_API_KEY",
        &config.tomorrow_weather_api_key_override,
        &config.tomorrow_weather_vault_secret_id_override,
        Some(ProviderSecretId::TomorrowIoApiKey),
    )
    .ok_or_else(|| {
        RealtimeError::new(
            TOMORROW_IO_PROVIDER_ID,
            RealtimeErrorKind::ProviderUnconfigured,
            None,
            "missing tomorrow.io realtime weather api key configuration",
            0,
        )
    })?;

    let mut url = Url::parse(config.tomorrow_weather_endpoint.as_str()).map_err(|error| {
        RealtimeError::new(
            TOMORROW_IO_PROVIDER_ID,
            RealtimeErrorKind::PolicyViolation,
            None,
            format!("invalid tomorrow.io weather endpoint URL: {}", error),
            0,
        )
    })?;
    {
        let mut query_pairs = url.query_pairs_mut();
        query_pairs.append_pair("location", request.query.as_str());
        query_pairs.append_pair("units", "metric");
        query_pairs.append_pair("apikey", api_key.as_str());
    }

    let (payload, latency_ms) =
        fetch_json_with_caps(TOMORROW_IO_PROVIDER_ID, request, config, url.as_str(), &[])?;
    let normalized_payload = normalize_provider_payload(
        "tomorrow.io",
        request.query.as_str(),
        request.now_ms,
        payload,
    );

    Ok(RealtimeAdapterOutput {
        provider_id: TOMORROW_IO_PROVIDER_ID.to_string(),
        endpoint_url: url.to_string(),
        title: "Tomorrow.io Realtime Weather".to_string(),
        trust_tier: "high".to_string(),
        retrieved_at_ms: request.now_ms,
        latency_ms,
        payload: normalized_payload,
    })
}

fn execute_weather_api(
    request: &ParsedRealtimeToolRequest,
    config: &RealtimeRuntimeConfig,
) -> Result<RealtimeAdapterOutput, RealtimeError> {
    let api_key = resolve_api_key(
        "SELENE_REALTIME_WEATHER_API_KEY",
        &config.weather_api_key_override,
        &config.weather_vault_secret_id_override,
        Some(ProviderSecretId::WeatherApiKey),
    )
    .ok_or_else(|| {
        RealtimeError::new(
            WEATHER_API_PROVIDER_ID,
            RealtimeErrorKind::ProviderUnconfigured,
            None,
            "missing secondary weatherapi.com realtime api key configuration",
            0,
        )
    })?;

    let mut url = Url::parse(config.weather_endpoint.as_str()).map_err(|error| {
        RealtimeError::new(
            WEATHER_API_PROVIDER_ID,
            RealtimeErrorKind::PolicyViolation,
            None,
            format!("invalid weather endpoint URL: {}", error),
            0,
        )
    })?;
    {
        let mut query_pairs = url.query_pairs_mut();
        query_pairs.append_pair("q", request.query.as_str());
        query_pairs.append_pair("key", api_key.as_str());
    }

    let (payload, latency_ms) =
        fetch_json_with_caps(WEATHER_API_PROVIDER_ID, request, config, url.as_str(), &[])?;
    let normalized_payload = normalize_provider_payload(
        "weatherapi.com",
        request.query.as_str(),
        request.now_ms,
        payload,
    );

    Ok(RealtimeAdapterOutput {
        provider_id: WEATHER_API_PROVIDER_ID.to_string(),
        endpoint_url: url.to_string(),
        title: "WeatherAPI.com Realtime Weather".to_string(),
        trust_tier: "medium".to_string(),
        retrieved_at_ms: request.now_ms,
        latency_ms,
        payload: normalized_payload,
    })
}

fn normalize_provider_payload(
    provider: &str,
    query: &str,
    retrieved_at_ms: i64,
    provider_payload: Value,
) -> Value {
    json!({
        "retrieved_at_ms": retrieved_at_ms,
        "trust_tier": if provider == "tomorrow.io" { "high" } else { "medium" },
        "provider": provider,
        "query": query,
        "provider_payload": provider_payload
    })
}
