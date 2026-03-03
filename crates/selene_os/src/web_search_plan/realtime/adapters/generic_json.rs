#![forbid(unsafe_code)]

use crate::web_search_plan::realtime::adapters::{
    fetch_json_with_caps, parse_retrieved_at_ms, parse_trust_tier, resolve_api_key,
    RealtimeAdapterOutput,
};
use crate::web_search_plan::realtime::{
    ParsedRealtimeToolRequest, RealtimeError, RealtimeErrorKind, RealtimeRuntimeConfig,
};
use url::Url;

const ADAPTER_ID: &str = "RealtimeGenericJson";

pub fn execute(
    request: &ParsedRealtimeToolRequest,
    config: &RealtimeRuntimeConfig,
) -> Result<RealtimeAdapterOutput, RealtimeError> {
    let url = parse_query_url(request.query.as_str())?;
    let api_key = resolve_api_key(
        "SELENE_REALTIME_GENERIC_API_KEY",
        &config.generic_api_key_override,
        &config.generic_vault_secret_id_override,
        None,
    )
    .ok_or_else(|| {
        RealtimeError::new(
            ADAPTER_ID,
            RealtimeErrorKind::ProviderUnconfigured,
            None,
            "missing generic realtime api key configuration",
            0,
        )
    })?;

    let authorization = format!("Bearer {}", api_key);
    let headers = [("Authorization", authorization.as_str())];
    let (payload, latency_ms) = fetch_json_with_caps(ADAPTER_ID, request, config, url.as_str(), &headers)?;
    let retrieved_at_ms = parse_retrieved_at_ms(ADAPTER_ID, &payload)?;
    let trust_tier = parse_trust_tier(&payload);

    let title = payload
        .get("title")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("Generic Real-Time API")
        .to_string();

    Ok(RealtimeAdapterOutput {
        provider_id: ADAPTER_ID.to_string(),
        endpoint_url: url.to_string(),
        title,
        trust_tier,
        retrieved_at_ms,
        latency_ms,
        payload,
    })
}

fn parse_query_url(query: &str) -> Result<Url, RealtimeError> {
    let trimmed = query.trim();
    if !trimmed.starts_with("http://") && !trimmed.starts_with("https://") {
        return Err(RealtimeError::new(
            ADAPTER_ID,
            RealtimeErrorKind::InvalidInput,
            None,
            "generic realtime query must be explicit URL",
            0,
        ));
    }
    Url::parse(trimmed).map_err(|error| {
        RealtimeError::new(
            ADAPTER_ID,
            RealtimeErrorKind::InvalidInput,
            None,
            format!("invalid realtime URL query: {}", error),
            0,
        )
    })
}
