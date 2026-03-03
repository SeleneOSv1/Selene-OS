#![forbid(unsafe_code)]

use serde_json::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DomainHint {
    CompanyRegistry,
    Filings,
    Patents,
    Academic,
    Pricing,
    GovDataset,
    GenericHttpJson,
}

impl DomainHint {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::CompanyRegistry => "company_registry",
            Self::Filings => "filings",
            Self::Patents => "patents",
            Self::Academic => "academic",
            Self::Pricing => "pricing",
            Self::GovDataset => "gov_dataset",
            Self::GenericHttpJson => "generic_http_json",
        }
    }

    pub fn parse(raw: &str) -> Option<Self> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "company_registry" => Some(Self::CompanyRegistry),
            "filings" => Some(Self::Filings),
            "patents" => Some(Self::Patents),
            "academic" => Some(Self::Academic),
            "pricing" => Some(Self::Pricing),
            "gov_dataset" => Some(Self::GovDataset),
            "generic_http_json" => Some(Self::GenericHttpJson),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdapterSelection {
    GenericHttpJson,
    GovDataset,
    CompanyRegistry,
    Filings,
    Patents,
    Academic,
    PricingProducts,
}

impl AdapterSelection {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::GenericHttpJson => "generic_http_json",
            Self::GovDataset => "gov_dataset",
            Self::CompanyRegistry => "company_registry",
            Self::Filings => "filings",
            Self::Patents => "patents",
            Self::Academic => "academic",
            Self::PricingProducts => "pricing_products",
        }
    }
}

pub fn extract_domain_hint(tool_request_packet: &Value) -> Option<DomainHint> {
    let budgets_hint = tool_request_packet
        .get("budgets")
        .and_then(Value::as_object)
        .and_then(|budgets| budgets.get("domain_hint"))
        .and_then(Value::as_str)
        .and_then(DomainHint::parse);

    if budgets_hint.is_some() {
        return budgets_hint;
    }

    tool_request_packet
        .get("query")
        .and_then(Value::as_str)
        .and_then(parse_inline_domain_hint)
}

pub fn route_adapter(
    query: &str,
    domain_hint: Option<DomainHint>,
) -> Result<AdapterSelection, String> {
    if let Some(hint) = domain_hint {
        return Ok(match hint {
            DomainHint::CompanyRegistry => AdapterSelection::CompanyRegistry,
            DomainHint::Filings => AdapterSelection::Filings,
            DomainHint::Patents => AdapterSelection::Patents,
            DomainHint::Academic => AdapterSelection::Academic,
            DomainHint::Pricing => AdapterSelection::PricingProducts,
            DomainHint::GovDataset => AdapterSelection::GovDataset,
            DomainHint::GenericHttpJson => AdapterSelection::GenericHttpJson,
        });
    }

    if looks_like_url(query) {
        Ok(AdapterSelection::GenericHttpJson)
    } else {
        Err(
            "structured routing requires domain_hint or explicit URL query; clarification needed"
                .to_string(),
        )
    }
}

fn parse_inline_domain_hint(query: &str) -> Option<DomainHint> {
    let normalized = query.trim();
    let marker = "domain_hint:";
    let start = normalized.to_ascii_lowercase();
    let marker_index = start.find(marker)?;
    let tail = &normalized[marker_index + marker.len()..];
    let hint_raw = tail
        .split_whitespace()
        .next()
        .unwrap_or_default()
        .trim_matches(|ch: char| ch == ';' || ch == ',');
    DomainHint::parse(hint_raw)
}

fn looks_like_url(query: &str) -> bool {
    let trimmed = query.trim();
    trimmed.starts_with("http://") || trimmed.starts_with("https://")
}
