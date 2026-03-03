#![forbid(unsafe_code)]

use serde_json::Value;

pub const DOMAIN_SELECTOR_VERSION: &str = "1.0.0";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RealtimeDomain {
    Weather,
    Finance,
    Flights,
    GenericRealTime,
}

impl RealtimeDomain {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Weather => "weather",
            Self::Finance => "finance",
            Self::Flights => "flights",
            Self::GenericRealTime => "generic_real_time",
        }
    }

    pub const fn provider_id(self) -> &'static str {
        match self {
            Self::Weather => "RealtimeWeather",
            Self::Finance => "RealtimeFinance",
            Self::Flights => "RealtimeFlights",
            Self::GenericRealTime => "RealtimeGenericJson",
        }
    }

    pub const fn title(self) -> &'static str {
        match self {
            Self::Weather => "Weather API",
            Self::Finance => "Finance API",
            Self::Flights => "Flights API",
            Self::GenericRealTime => "Generic Real-Time API",
        }
    }

    pub fn parse(raw: &str) -> Option<Self> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "weather" => Some(Self::Weather),
            "finance" => Some(Self::Finance),
            "flights" => Some(Self::Flights),
            "generic_real_time" | "generic_realtime" | "generic" => Some(Self::GenericRealTime),
            _ => None,
        }
    }
}

pub fn detect_domain(query: &str, explicit_hint: Option<&str>) -> RealtimeDomain {
    if let Some(hint) = explicit_hint.and_then(RealtimeDomain::parse) {
        return hint;
    }

    let normalized = query.trim().to_ascii_lowercase();
    if contains_any(
        normalized.as_str(),
        &[
            "weather",
            "temperature",
            "forecast",
            "humidity",
            "rain",
            "wind",
            "snow",
        ],
    ) {
        RealtimeDomain::Weather
    } else if contains_any(
        normalized.as_str(),
        &[
            "stock",
            "ticker",
            "quote",
            "market",
            "price",
            "crypto",
            "forex",
            "fx",
            "equity",
        ],
    ) {
        RealtimeDomain::Finance
    } else if contains_any(
        normalized.as_str(),
        &[
            "flight",
            "airline",
            "departure",
            "arrival",
            "gate",
            "boarding",
            "iata",
        ],
    ) {
        RealtimeDomain::Flights
    } else {
        RealtimeDomain::GenericRealTime
    }
}

pub fn extract_domain_hint(tool_request_packet: &Value) -> Option<String> {
    let from_budgets = tool_request_packet
        .get("budgets")
        .and_then(Value::as_object)
        .and_then(|budgets| budgets.get("domain_hint"))
        .and_then(Value::as_str)
        .map(|raw| raw.trim().to_string())
        .filter(|value| !value.is_empty());

    if from_budgets.is_some() {
        return from_budgets;
    }

    let query = tool_request_packet.get("query").and_then(Value::as_str)?;
    parse_inline_domain_hint(query)
}

fn parse_inline_domain_hint(query: &str) -> Option<String> {
    let marker = "domain_hint:";
    let lower = query.to_ascii_lowercase();
    let index = lower.find(marker)?;
    let tail = &query[index + marker.len()..];
    let hint = tail
        .split_whitespace()
        .next()
        .unwrap_or_default()
        .trim_matches(|ch: char| ch == ';' || ch == ',')
        .trim()
        .to_string();
    if hint.is_empty() { None } else { Some(hint) }
}

fn contains_any(haystack: &str, needles: &[&str]) -> bool {
    needles.iter().any(|needle| haystack.contains(needle))
}
