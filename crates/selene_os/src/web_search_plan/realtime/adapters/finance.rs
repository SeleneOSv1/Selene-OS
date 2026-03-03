#![forbid(unsafe_code)]

use crate::web_search_plan::realtime::adapters::{
    fetch_json_with_caps, parse_retrieved_at_ms, parse_trust_tier, resolve_api_key,
    RealtimeAdapterOutput,
};
use crate::web_search_plan::realtime::{
    ParsedRealtimeToolRequest, RealtimeError, RealtimeErrorKind, RealtimeRuntimeConfig,
};
use url::Url;

const ADAPTER_ID: &str = "RealtimeFinance";

pub fn execute(
    request: &ParsedRealtimeToolRequest,
    config: &RealtimeRuntimeConfig,
) -> Result<RealtimeAdapterOutput, RealtimeError> {
    let api_key = resolve_api_key(
        "SELENE_REALTIME_FINANCE_API_KEY",
        &config.finance_api_key_override,
        &config.finance_vault_secret_id_override,
        None,
    )
    .ok_or_else(|| {
        RealtimeError::new(
            ADAPTER_ID,
            RealtimeErrorKind::ProviderUnconfigured,
            None,
            "missing finance realtime api key configuration",
            0,
        )
    })?;

    let mut url = Url::parse(config.finance_endpoint.as_str()).map_err(|error| {
        RealtimeError::new(
            ADAPTER_ID,
            RealtimeErrorKind::PolicyViolation,
            None,
            format!("invalid finance endpoint URL: {}", error),
            0,
        )
    })?;
    {
        let mut query_pairs = url.query_pairs_mut();
        query_pairs.append_pair("query", request.query.as_str());
        query_pairs.append_pair("apikey", api_key.as_str());
    }

    let (payload, latency_ms) = fetch_json_with_caps(ADAPTER_ID, request, config, url.as_str(), &[])?;
    let retrieved_at_ms = parse_retrieved_at_ms(ADAPTER_ID, &payload)?;
    let trust_tier = parse_trust_tier(&payload);

    Ok(RealtimeAdapterOutput {
        provider_id: ADAPTER_ID.to_string(),
        endpoint_url: url.to_string(),
        title: "Finance API".to_string(),
        trust_tier,
        retrieved_at_ms,
        latency_ms,
        payload,
    })
}
