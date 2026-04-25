#![forbid(unsafe_code)]

pub mod adapters;
pub mod domains;
pub mod freshness;
pub mod ttl_policy;

use crate::web_search_plan::perf_cost::tiers::ImportanceTier;
use crate::web_search_plan::proxy::proxy_config::{ProxyConfig, SystemEnvProvider};
use crate::web_search_plan::proxy::ProxyMode;
use crate::web_search_plan::realtime::adapters::{run_adapter, RealtimeAdapterOutput};
use crate::web_search_plan::realtime::domains::{
    detect_domain, extract_domain_hint, RealtimeDomain, DOMAIN_SELECTOR_VERSION,
};
use crate::web_search_plan::realtime::freshness::evaluate;
use crate::web_search_plan::realtime::ttl_policy::{ttl_ms, REALTIME_TTL_POLICY_VERSION};
use serde_json::{json, Map, Value};

pub const REALTIME_SCHEMA_VERSION: &str = "1.0.0";
pub const REALTIME_ENGINE_ID: &str = "PH1.E";
pub const REALTIME_USER_AGENT: &str = "selene-realtime/1.0";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RealtimeErrorKind {
    ProviderUnconfigured,
    ProviderUpstreamFailed,
    TimeoutExceeded,
    StaleData,
    FreshnessPolicyUnmet,
    PolicyViolation,
    InvalidInput,
}

impl RealtimeErrorKind {
    pub const fn reason_code(self) -> &'static str {
        match self {
            Self::ProviderUnconfigured => "provider_unconfigured",
            Self::ProviderUpstreamFailed => "provider_upstream_failed",
            Self::TimeoutExceeded => "timeout_exceeded",
            Self::StaleData => "stale_data",
            Self::FreshnessPolicyUnmet => "freshness_policy_unmet",
            Self::PolicyViolation | Self::InvalidInput => "policy_violation",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RealtimeError {
    pub adapter_id: String,
    pub kind: RealtimeErrorKind,
    pub status_code: Option<u16>,
    pub message: String,
    pub latency_ms: u64,
}

impl RealtimeError {
    pub fn new(
        adapter_id: impl Into<String>,
        kind: RealtimeErrorKind,
        status_code: Option<u16>,
        message: impl Into<String>,
        latency_ms: u64,
    ) -> Self {
        Self {
            adapter_id: adapter_id.into(),
            kind,
            status_code,
            message: message.into(),
            latency_ms,
        }
    }

    pub const fn reason_code(&self) -> &'static str {
        self.kind.reason_code()
    }
}

#[derive(Debug, Clone)]
pub struct RealtimeRuntimeConfig {
    pub timeout_ms: u64,
    pub max_response_bytes: usize,
    pub user_agent: String,

    pub generic_api_key_override: Option<String>,
    pub generic_vault_secret_id_override: Option<String>,

    pub weather_endpoint: String,
    pub weather_api_key_override: Option<String>,
    pub weather_vault_secret_id_override: Option<String>,
    pub tomorrow_weather_endpoint: String,
    pub tomorrow_weather_api_key_override: Option<String>,
    pub tomorrow_weather_vault_secret_id_override: Option<String>,

    pub finance_endpoint: String,
    pub finance_api_key_override: Option<String>,
    pub finance_vault_secret_id_override: Option<String>,

    pub flights_endpoint: String,
    pub flights_api_key_override: Option<String>,
    pub flights_vault_secret_id_override: Option<String>,
}

impl Default for RealtimeRuntimeConfig {
    fn default() -> Self {
        Self {
            timeout_ms: 2_000,
            max_response_bytes: 512 * 1024,
            user_agent: REALTIME_USER_AGENT.to_string(),

            generic_api_key_override: None,
            generic_vault_secret_id_override: std::env::var("SELENE_REALTIME_GENERIC_VAULT_SECRET_ID")
                .ok(),

            weather_endpoint: std::env::var("SELENE_REALTIME_WEATHER_ENDPOINT")
                .unwrap_or_else(|_| "https://api.weatherapi.com/v1/current.json".to_string()),
            weather_api_key_override: None,
            weather_vault_secret_id_override: std::env::var(
                "SELENE_REALTIME_WEATHER_VAULT_SECRET_ID",
            )
            .ok(),
            tomorrow_weather_endpoint: std::env::var("SELENE_REALTIME_TOMORROW_IO_ENDPOINT")
                .unwrap_or_else(|_| "https://api.tomorrow.io/v4/weather/realtime".to_string()),
            tomorrow_weather_api_key_override: None,
            tomorrow_weather_vault_secret_id_override: std::env::var(
                "SELENE_REALTIME_TOMORROW_IO_VAULT_SECRET_ID",
            )
            .ok(),

            finance_endpoint: std::env::var("SELENE_REALTIME_FINANCE_ENDPOINT")
                .unwrap_or_else(|_| "https://api.finance.example.com/quote".to_string()),
            finance_api_key_override: None,
            finance_vault_secret_id_override: std::env::var(
                "SELENE_REALTIME_FINANCE_VAULT_SECRET_ID",
            )
            .ok(),

            flights_endpoint: std::env::var("SELENE_REALTIME_FLIGHTS_ENDPOINT")
                .unwrap_or_else(|_| "https://api.flights.example.com/status".to_string()),
            flights_api_key_override: None,
            flights_vault_secret_id_override: std::env::var(
                "SELENE_REALTIME_FLIGHTS_VAULT_SECRET_ID",
            )
            .ok(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ParsedRealtimeToolRequest {
    pub trace_id: String,
    pub query: String,
    pub created_at_ms: i64,
    pub now_ms: i64,
    pub intended_consumers: Vec<String>,
    pub importance_tier: String,
    pub budgets: Value,
    pub domain: RealtimeDomain,
    pub proxy_config: ProxyConfig,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RealtimeConnectorResult {
    pub domain: RealtimeDomain,
    pub evidence_packet: Value,
    pub ttl_ms: u64,
    pub age_ms: i64,
    pub freshness_score: f64,
    pub stale: bool,
}

pub fn execute_realtime_from_tool_request(
    tool_request_packet: &Value,
    now_ms: i64,
    config: &RealtimeRuntimeConfig,
) -> Result<RealtimeConnectorResult, RealtimeError> {
    let request = parse_tool_request_packet(tool_request_packet, now_ms)?;
    let output = run_adapter(&request, config)?;
    finalize_realtime_result(&request, output)
}

pub(crate) fn finalize_realtime_result(
    request: &ParsedRealtimeToolRequest,
    output: RealtimeAdapterOutput,
) -> Result<RealtimeConnectorResult, RealtimeError> {
    let tier = ImportanceTier::parse_or_default(request.importance_tier.as_str());
    let applied_ttl_ms = ttl_ms(request.domain, tier);
    let freshness = evaluate(request.now_ms, output.retrieved_at_ms, applied_ttl_ms).map_err(|error| {
        RealtimeError::new(
            output.provider_id.as_str(),
            RealtimeErrorKind::PolicyViolation,
            None,
            format!("freshness evaluation failed: {}", error),
            output.latency_ms,
        )
    })?;

    if freshness.stale {
        return Err(RealtimeError::new(
            output.provider_id,
            RealtimeErrorKind::StaleData,
            None,
            "realtime payload is stale beyond ttl",
            output.latency_ms,
        ));
    }

    let evidence_packet = json!({
        "schema_version": REALTIME_SCHEMA_VERSION,
        "produced_by": REALTIME_ENGINE_ID,
        "intended_consumers": request.intended_consumers,
        "created_at_ms": request.created_at_ms,
        "trace_id": request.trace_id,
        "query": request.query,
        "retrieved_at_ms": output.retrieved_at_ms,
        "provider_runs": [
            {
                "provider_id": output.provider_id,
                "endpoint": "real_time",
                "latency_ms": output.latency_ms,
                "error": Value::Null,
                "ttl_ms_applied": applied_ttl_ms,
                "stale": false
            }
        ],
        "sources": [
            {
                "title": output.title,
                "url": output.endpoint_url,
                "snippet": "realtime adapter response",
                "media_type": "structured",
                "provider_id": request.domain.provider_id(),
                "rank": 1,
                "published_at": Value::Null,
                "freshness_score": freshness.freshness_score,
                "trust_tier": output.trust_tier
            }
        ],
        "content_chunks": [],
        "trust_metadata": {
            "realtime": {
                "domain": request.domain.as_str(),
                "domain_selector_version": DOMAIN_SELECTOR_VERSION,
                "ttl_policy_version": REALTIME_TTL_POLICY_VERSION,
                "ttl_ms": applied_ttl_ms,
                "age_ms": freshness.age_ms,
                "freshness_score": freshness.freshness_score,
                "stale": false,
                "trust_tier": output.trust_tier,
                "payload": output.payload
            }
        }
    });

    Ok(RealtimeConnectorResult {
        domain: request.domain,
        evidence_packet,
        ttl_ms: applied_ttl_ms,
        age_ms: freshness.age_ms,
        freshness_score: freshness.freshness_score,
        stale: false,
    })
}

pub fn append_realtime_audit_fields(
    audit_packet: &mut Value,
    result: &RealtimeConnectorResult,
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
        *transition_value = json!({ "state": state });
        transition_value
            .as_object_mut()
            .ok_or_else(|| "turn_state_transition conversion failed".to_string())?
    } else {
        return Err("turn_state_transition must be string or object".to_string());
    };

    transition_obj.insert(
        "realtime_audit".to_string(),
        json!({
            "domain": result.domain.as_str(),
            "ttl_ms": result.ttl_ms,
            "age_ms": result.age_ms,
            "freshness_score": result.freshness_score,
            "stale": result.stale,
            "reason_code": if result.stale {
                Value::String("stale_data".to_string())
            } else {
                Value::Null
            },
        }),
    );

    Ok(())
}

fn parse_tool_request_packet(
    tool_request_packet: &Value,
    now_ms: i64,
) -> Result<ParsedRealtimeToolRequest, RealtimeError> {
    let obj = tool_request_packet.as_object().ok_or_else(|| {
        RealtimeError::new(
            "realtime_parser",
            RealtimeErrorKind::InvalidInput,
            None,
            "tool request packet must be object",
            0,
        )
    })?;

    let mode = obj.get("mode").and_then(Value::as_str).unwrap_or_default();
    if mode != "real_time" && mode != "structured" {
        return Err(RealtimeError::new(
            "realtime_parser",
            RealtimeErrorKind::InvalidInput,
            None,
            format!(
                "tool request mode must be real_time or structured for realtime lane, got {}",
                mode
            ),
            0,
        ));
    }

    let query = obj
        .get("query")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| {
            RealtimeError::new(
                "realtime_parser",
                RealtimeErrorKind::InvalidInput,
                None,
                "tool request query missing",
                0,
            )
        })?
        .to_string();

    let trace_id = obj
        .get("trace_id")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| {
            RealtimeError::new(
                "realtime_parser",
                RealtimeErrorKind::InvalidInput,
                None,
                "tool request trace_id missing",
                0,
            )
        })?
        .to_string();

    let created_at_ms = obj
        .get("created_at_ms")
        .and_then(Value::as_i64)
        .unwrap_or(now_ms);

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

    let importance_tier = obj
        .get("importance_tier")
        .and_then(Value::as_str)
        .unwrap_or("medium")
        .to_string();
    let budgets = obj.get("budgets").cloned().unwrap_or_else(|| json!({}));
    let explicit_hint = extract_domain_hint(tool_request_packet);
    let domain = detect_domain(query.as_str(), explicit_hint.as_deref());

    let env = SystemEnvProvider;
    let proxy_mode_raw =
        std::env::var("SELENE_REALTIME_PROXY_MODE").unwrap_or_else(|_| "off".to_string());
    let proxy_mode = ProxyMode::parse(&proxy_mode_raw).unwrap_or(ProxyMode::Off);
    let proxy_config = ProxyConfig::from_env(proxy_mode, &env);

    Ok(ParsedRealtimeToolRequest {
        trace_id,
        query,
        created_at_ms,
        now_ms,
        intended_consumers,
        importance_tier,
        budgets,
        domain,
        proxy_config,
    })
}

#[cfg(test)]
pub mod realtime_tests;
