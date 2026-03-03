#![forbid(unsafe_code)]

use crate::web_search_plan::structured::adapters::fetch_json_with_caps;
use crate::web_search_plan::structured::schema::StructuredSchemaId;
use crate::web_search_plan::structured::types::{
    StructuredAdapterOutput, StructuredAdapterRequest, StructuredConnectorError,
    StructuredErrorKind, StructuredRow, StructuredRuntimeConfig, StructuredValue,
    STRUCTURED_SCHEMA_VERSION,
};
use serde_json::{json, Value};

pub const ADAPTER_ID: &str = "generic_http_json";
const MAX_ROWS: usize = 256;

pub fn execute(
    request: &StructuredAdapterRequest,
    config: &StructuredRuntimeConfig,
) -> Result<StructuredAdapterOutput, StructuredConnectorError> {
    let url = extract_url_from_query(request.query.as_str()).ok_or_else(|| {
        StructuredConnectorError::new(
            ADAPTER_ID,
            StructuredErrorKind::InsufficientEvidence,
            None,
            "structured generic_http_json requires explicit URL query",
            0,
        )
    })?;

    let (payload, latency_ms) = fetch_json_with_caps(ADAPTER_ID, request, config, url, &[])?;
    let mut rows = Vec::new();
    flatten_json_to_rows(
        "root",
        "root",
        &payload,
        url,
        &mut rows,
        &mut Vec::new(),
        MAX_ROWS,
    );

    if rows.is_empty() {
        return Err(StructuredConnectorError::new(
            ADAPTER_ID,
            StructuredErrorKind::EmptyResults,
            None,
            "structured generic_http_json returned no scalar values",
            latency_ms,
        ));
    }

    let row_count = rows.len();

    Ok(StructuredAdapterOutput {
        schema_id: StructuredSchemaId::GenericHttpJsonV1.as_str().to_string(),
        rows,
        provider_runs: vec![json!({
            "provider_id": ADAPTER_ID,
            "endpoint": "structured",
            "latency_ms": latency_ms,
            "results_count": row_count,
            "error": Value::Null,
        })],
        sources: vec![json!({
            "title": "Structured JSON Source",
            "url": url,
            "snippet": "structured connector fetched JSON payload",
            "media_type": "structured",
            "provider_id": ADAPTER_ID,
            "rank": 1,
            "canonical_url": url.to_ascii_lowercase(),
        })],
        errors: Vec::new(),
    })
}

pub fn extract_url_from_query(query: &str) -> Option<&str> {
    let trimmed = query.trim();
    if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
        return Some(trimmed);
    }

    let normalized = trimmed.to_ascii_lowercase();
    let marker = "url=";
    let marker_index = normalized.find(marker)?;
    let tail = &trimmed[marker_index + marker.len()..];
    tail.split_whitespace()
        .next()
        .map(str::trim)
        .filter(|value| value.starts_with("http://") || value.starts_with("https://"))
}

fn flatten_json_to_rows(
    entity: &str,
    attribute_prefix: &str,
    value: &Value,
    source_url: &str,
    rows: &mut Vec<StructuredRow>,
    path_stack: &mut Vec<String>,
    max_rows: usize,
) {
    if rows.len() >= max_rows {
        return;
    }

    match value {
        Value::Object(map) => {
            let mut keys = map.keys().cloned().collect::<Vec<String>>();
            keys.sort();
            for key in keys {
                if rows.len() >= max_rows {
                    return;
                }
                if let Some(child) = map.get(key.as_str()) {
                    path_stack.push(key.clone());
                    let attribute = if attribute_prefix.is_empty() {
                        key.clone()
                    } else {
                        format!("{}.{}", attribute_prefix, key)
                    };
                    flatten_json_to_rows(
                        entity,
                        attribute.as_str(),
                        child,
                        source_url,
                        rows,
                        path_stack,
                        max_rows,
                    );
                    let _ = path_stack.pop();
                }
            }
        }
        Value::Array(items) => {
            for (index, child) in items.iter().enumerate() {
                if rows.len() >= max_rows {
                    return;
                }
                path_stack.push(index.to_string());
                let attribute = format!("{}[{}]", attribute_prefix, index);
                flatten_json_to_rows(
                    entity,
                    attribute.as_str(),
                    child,
                    source_url,
                    rows,
                    path_stack,
                    max_rows,
                );
                let _ = path_stack.pop();
            }
        }
        _ => {
            if let Some(typed_value) = infer_typed_value(value) {
                rows.push(StructuredRow {
                    entity: entity.to_string(),
                    attribute: attribute_prefix.to_string(),
                    value: typed_value,
                    unit: None,
                    as_of_ms: None,
                    source_url: source_url.to_string(),
                    source_ref: source_url.to_string(),
                    confidence: None,
                    schema_version: STRUCTURED_SCHEMA_VERSION.to_string(),
                });
            }
        }
    }
}

fn infer_typed_value(value: &Value) -> Option<StructuredValue> {
    match value {
        Value::Null => None,
        Value::Bool(value) => Some(StructuredValue::Bool { value: *value }),
        Value::Number(number) => {
            if let Some(value) = number.as_i64() {
                Some(StructuredValue::Int { value })
            } else {
                number
                    .as_f64()
                    .map(|value| StructuredValue::Float { value })
            }
        }
        Value::String(text) => parse_string_typed_value(text),
        Value::Array(_) | Value::Object(_) => None,
    }
}

fn parse_string_typed_value(text: &str) -> Option<StructuredValue> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return None;
    }

    if let Some(percent) = parse_percent(trimmed) {
        return Some(StructuredValue::Percent { value: percent });
    }
    if let Some((amount, currency_code)) = parse_currency(trimmed) {
        return Some(StructuredValue::Currency {
            amount,
            currency_code,
        });
    }
    if is_date_like(trimmed) {
        return Some(StructuredValue::Date {
            value: trimmed.to_string(),
        });
    }

    Some(StructuredValue::String {
        value: trimmed.to_string(),
    })
}

fn parse_percent(text: &str) -> Option<f64> {
    let percent = text.strip_suffix('%')?.trim().parse::<f64>().ok()?;
    if percent.is_finite() && (0.0..=100.0).contains(&percent) {
        Some(percent)
    } else {
        None
    }
}

fn parse_currency(text: &str) -> Option<(f64, String)> {
    if let Some(amount) = text
        .strip_prefix('$')
        .and_then(|raw| raw.parse::<f64>().ok())
    {
        if amount.is_finite() {
            return Some((amount, "USD".to_string()));
        }
    }

    let mut parts = text.split_whitespace().collect::<Vec<&str>>();
    if parts.len() == 2 {
        let first = parts.remove(0);
        let second = parts.remove(0);

        if second.len() == 3 && second.chars().all(|ch| ch.is_ascii_alphabetic()) {
            if let Ok(amount) = first.parse::<f64>() {
                if amount.is_finite() {
                    return Some((amount, second.to_ascii_uppercase()));
                }
            }
        }
    }

    None
}

fn is_date_like(text: &str) -> bool {
    let bytes = text.as_bytes();
    if bytes.len() >= 10
        && bytes.get(4) == Some(&b'-')
        && bytes.get(7) == Some(&b'-')
        && text[0..4].chars().all(|ch| ch.is_ascii_digit())
        && text[5..7].chars().all(|ch| ch.is_ascii_digit())
        && text[8..10].chars().all(|ch| ch.is_ascii_digit())
    {
        return true;
    }
    text.chars().all(|ch| ch.is_ascii_digit()) && text.parse::<i64>().is_ok()
}
