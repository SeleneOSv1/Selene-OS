#![forbid(unsafe_code)]

use serde_json::Value;
use url::Url;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum JurisdictionConfidence {
    High,
    Medium,
    Low,
}

impl JurisdictionConfidence {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::High => "high",
            Self::Medium => "medium",
            Self::Low => "low",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JurisdictionResolution {
    pub jurisdiction_code: String,
    pub confidence: JurisdictionConfidence,
    pub method: String,
}

pub fn resolve_jurisdiction(
    tool_request_packet: &Value,
    evidence_packet: &Value,
) -> Option<JurisdictionResolution> {
    if let Some(code) = explicit_jurisdiction(tool_request_packet) {
        return Some(JurisdictionResolution {
            jurisdiction_code: code,
            confidence: JurisdictionConfidence::High,
            method: "explicit".to_string(),
        });
    }

    if let Some(code) = query_keyword_jurisdiction(tool_request_packet) {
        return Some(JurisdictionResolution {
            jurisdiction_code: code,
            confidence: JurisdictionConfidence::Medium,
            method: "query_keyword".to_string(),
        });
    }

    if let Some(code) = source_url_jurisdiction(evidence_packet) {
        return Some(JurisdictionResolution {
            jurisdiction_code: code,
            confidence: JurisdictionConfidence::Low,
            method: "source_url".to_string(),
        });
    }

    None
}

pub fn map_url_to_jurisdiction(url: &str) -> Option<String> {
    let host = Url::parse(url)
        .ok()
        .and_then(|parsed| parsed.host_str().map(|value| value.to_ascii_lowercase()))?;
    map_host_to_jurisdiction(host.as_str())
}

fn explicit_jurisdiction(tool_request_packet: &Value) -> Option<String> {
    let from_budgets = tool_request_packet
        .get("budgets")
        .and_then(Value::as_object)
        .and_then(|budgets| {
            budgets
                .get("jurisdiction")
                .and_then(Value::as_str)
                .or_else(|| budgets.get("jurisdiction_code").and_then(Value::as_str))
        })
        .and_then(normalize_jurisdiction_code);
    if from_budgets.is_some() {
        return from_budgets;
    }

    tool_request_packet
        .get("jurisdiction")
        .and_then(Value::as_str)
        .and_then(normalize_jurisdiction_code)
}

fn query_keyword_jurisdiction(tool_request_packet: &Value) -> Option<String> {
    let query = tool_request_packet
        .get("query")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_ascii_lowercase();
    if query.is_empty() {
        return None;
    }

    let keywords = [
        ("singapore", "SG"),
        ("mas", "SG"),
        ("us-ca", "US-CA"),
        ("california", "US-CA"),
        ("sec", "US"),
        ("united states", "US"),
        ("germany", "DE"),
        ("bafin", "DE"),
        ("european union", "EU"),
        ("eu", "EU"),
    ];

    for (keyword, code) in keywords {
        if query.contains(keyword) {
            return Some(code.to_string());
        }
    }
    None
}

fn source_url_jurisdiction(evidence_packet: &Value) -> Option<String> {
    let sources = evidence_packet.get("sources").and_then(Value::as_array)?;
    for source in sources {
        if let Some(explicit) = source
            .get("jurisdiction_code")
            .and_then(Value::as_str)
            .and_then(normalize_jurisdiction_code)
        {
            return Some(explicit);
        }
        if let Some(url) = source.get("url").and_then(Value::as_str) {
            if let Some(code) = map_url_to_jurisdiction(url) {
                return Some(code);
            }
        }
    }
    None
}

fn map_host_to_jurisdiction(host: &str) -> Option<String> {
    if host.ends_with(".gov.sg") || host.contains("mas.gov.sg") || host.contains("gov.sg") {
        return Some("SG".to_string());
    }
    if host == "sec.gov" || host.ends_with(".sec.gov") || host.ends_with(".gov") {
        return Some("US".to_string());
    }
    if host.ends_with(".ca.gov") || host.contains("ca.gov") {
        return Some("US-CA".to_string());
    }
    if host.ends_with(".de") || host.contains("bafin.de") {
        return Some("DE".to_string());
    }
    if host.ends_with(".eu") || host.contains("europa.eu") {
        return Some("EU".to_string());
    }
    None
}

fn normalize_jurisdiction_code(raw: &str) -> Option<String> {
    let normalized = raw.trim().to_ascii_uppercase();
    if normalized.is_empty() {
        return None;
    }
    match normalized.as_str() {
        "SG" | "SINGAPORE" => Some("SG".to_string()),
        "US" | "USA" | "UNITED STATES" => Some("US".to_string()),
        "US-CA" | "CA" | "CALIFORNIA" | "US_CA" => Some("US-CA".to_string()),
        "DE" | "GERMANY" => Some("DE".to_string()),
        "EU" | "EUR" | "EUROPEAN UNION" => Some("EU".to_string()),
        _ => Some(normalized),
    }
}
