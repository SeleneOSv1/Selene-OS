#![forbid(unsafe_code)]

use crate::web_search_plan::proxy::proxy_config::ProxyConfig;
use crate::web_search_plan::structured::registry::DomainHint;
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const STRUCTURED_SCHEMA_VERSION: &str = "1.0.0";
pub const STRUCTURED_ENGINE_ID: &str = "PH1.E";
pub const DEFAULT_STRUCTURED_TIMEOUT_MS: u64 = 2_000;
pub const DEFAULT_STRUCTURED_MAX_RESPONSE_BYTES: usize = 512 * 1024;
pub const DEFAULT_STRUCTURED_USER_AGENT: &str = "selene-structured-connectors/1.0";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum StructuredValue {
    String { value: String },
    Int { value: i64 },
    Float { value: f64 },
    Bool { value: bool },
    Date { value: String },
    Currency { amount: f64, currency_code: String },
    Percent { value: f64 },
}

impl StructuredValue {
    pub fn ordering_key(&self) -> String {
        match self {
            Self::String { value } => format!("string:{}", value),
            Self::Int { value } => format!("int:{:020}", value),
            Self::Float { value } => format!("float:{:.10}", value),
            Self::Bool { value } => format!("bool:{}", value),
            Self::Date { value } => format!("date:{}", value),
            Self::Currency {
                amount,
                currency_code,
            } => {
                format!("currency:{}:{:.10}", currency_code, amount)
            }
            Self::Percent { value } => format!("percent:{:.10}", value),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StructuredRow {
    pub entity: String,
    pub attribute: String,
    pub value: StructuredValue,
    pub unit: Option<String>,
    pub as_of_ms: Option<i64>,
    pub source_url: String,
    pub source_ref: String,
    pub confidence: Option<f64>,
    pub schema_version: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StructuredExtraction {
    pub query: String,
    pub rows: Vec<StructuredRow>,
    pub schema_id: String,
    pub extracted_at_ms: i64,
    pub provider_runs: Vec<Value>,
    pub sources: Vec<Value>,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StructuredErrorKind {
    ProviderUnconfigured,
    ProviderUpstreamFailed,
    TimeoutExceeded,
    EmptyResults,
    PolicyViolation,
    InsufficientEvidence,
    ParseFailed,
}

impl StructuredErrorKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ProviderUnconfigured => "provider_unconfigured",
            Self::ProviderUpstreamFailed => "provider_upstream_failed",
            Self::TimeoutExceeded => "timeout_exceeded",
            Self::EmptyResults => "empty_results",
            Self::PolicyViolation => "policy_violation",
            Self::InsufficientEvidence => "insufficient_evidence",
            Self::ParseFailed => "parse_failed",
        }
    }

    pub const fn reason_code(self) -> &'static str {
        match self {
            Self::ProviderUnconfigured => "provider_unconfigured",
            Self::ProviderUpstreamFailed | Self::ParseFailed => "provider_upstream_failed",
            Self::TimeoutExceeded => "timeout_exceeded",
            Self::EmptyResults => "empty_results",
            Self::PolicyViolation => "policy_violation",
            Self::InsufficientEvidence => "insufficient_evidence",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructuredConnectorError {
    pub adapter_id: String,
    pub kind: StructuredErrorKind,
    pub status_code: Option<u16>,
    pub message: String,
    pub latency_ms: u64,
}

impl StructuredConnectorError {
    pub fn new(
        adapter_id: impl Into<String>,
        kind: StructuredErrorKind,
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
pub struct StructuredAdapterRequest {
    pub trace_id: String,
    pub query: String,
    pub created_at_ms: i64,
    pub now_ms: i64,
    pub intended_consumers: Vec<String>,
    pub importance_tier: String,
    pub domain_hint: Option<DomainHint>,
    pub budgets: Value,
    pub proxy_config: ProxyConfig,
}

#[derive(Debug, Clone)]
pub struct StructuredAdapterOutput {
    pub schema_id: String,
    pub rows: Vec<StructuredRow>,
    pub provider_runs: Vec<Value>,
    pub sources: Vec<Value>,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct StructuredRuntimeConfig {
    pub timeout_ms: u64,
    pub max_response_bytes: usize,
    pub user_agent: String,
    pub gov_dataset_endpoint: String,
    pub gov_dataset_api_key_override: Option<String>,
    pub gov_dataset_vault_secret_id_override: Option<String>,
}

impl Default for StructuredRuntimeConfig {
    fn default() -> Self {
        Self {
            timeout_ms: DEFAULT_STRUCTURED_TIMEOUT_MS,
            max_response_bytes: DEFAULT_STRUCTURED_MAX_RESPONSE_BYTES,
            user_agent: DEFAULT_STRUCTURED_USER_AGENT.to_string(),
            gov_dataset_endpoint: std::env::var("SELENE_GOV_DATASET_ENDPOINT").unwrap_or_else(
                |_| "https://api.data.gov/ed/collegescorecard/v1/schools".to_string(),
            ),
            gov_dataset_api_key_override: None,
            gov_dataset_vault_secret_id_override: std::env::var(
                "SELENE_GOV_DATASET_VAULT_SECRET_ID",
            )
            .ok()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty()),
        }
    }
}
