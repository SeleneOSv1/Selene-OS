#![forbid(unsafe_code)]

use crate::web_search_plan::analytics::decimal::{decimal_to_string, round_decimal};
use crate::web_search_plan::contract_hash::sha256_hex;
use crate::web_search_plan::structured::types::{StructuredRow, StructuredValue};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TemporalValue {
    String { value: String },
    Int { value: i64 },
    Decimal { value: String },
    Bool { value: bool },
    Date { value: String },
    Currency { amount: String, currency_code: String },
    Percent { value: String },
}

impl TemporalValue {
    pub fn ordering_key(&self) -> String {
        match self {
            Self::String { value } => format!("string:{}", value),
            Self::Int { value } => format!("int:{:020}", value),
            Self::Decimal { value } => format!("decimal:{}", value),
            Self::Bool { value } => format!("bool:{}", value),
            Self::Date { value } => format!("date:{}", value),
            Self::Currency {
                amount,
                currency_code,
            } => format!("currency:{}:{}", currency_code, amount),
            Self::Percent { value } => format!("percent:{}", value),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TimelineEvent {
    pub event_id: String,
    pub entity: String,
    pub attribute: String,
    pub value: TemporalValue,
    pub as_of_ms: i64,
    pub source_refs: Vec<String>,
}

pub fn build_timeline_events(
    rows: &[StructuredRow],
    evidence_packet: &Value,
    include_source_events: bool,
) -> Vec<TimelineEvent> {
    let source_timestamps = extract_source_timestamps(evidence_packet);
    let mut events = Vec::new();

    for row in rows {
        let as_of_ms = row
            .as_of_ms
            .or_else(|| source_timestamps.get(row.source_url.as_str()).copied())
            .or_else(|| infer_date_ms_from_row(row));

        let Some(as_of_ms) = as_of_ms else {
            continue;
        };

        let value = structured_to_temporal_value(&row.value);
        let mut source_refs = vec![row.source_ref.clone(), row.source_url.clone()];
        source_refs.sort();
        source_refs.dedup();

        let event_id = derive_event_id(
            row.entity.as_str(),
            row.attribute.as_str(),
            &value,
            as_of_ms,
            source_refs.as_slice(),
        );
        events.push(TimelineEvent {
            event_id,
            entity: row.entity.clone(),
            attribute: row.attribute.clone(),
            value,
            as_of_ms,
            source_refs,
        });
    }

    if include_source_events {
        let mut source_items = source_timestamps
            .into_iter()
            .map(|(url, timestamp_ms)| (url.to_string(), timestamp_ms))
            .collect::<Vec<(String, i64)>>();
        source_items.sort();

        for (url, as_of_ms) in source_items {
            let source_refs = vec![url.clone()];
            let value = TemporalValue::String { value: url.clone() };
            let event_id = derive_event_id(
                "source",
                "published_at",
                &value,
                as_of_ms,
                source_refs.as_slice(),
            );
            events.push(TimelineEvent {
                event_id,
                entity: "source".to_string(),
                attribute: "published_at".to_string(),
                value,
                as_of_ms,
                source_refs,
            });
        }
    }

    events.sort_by(|left, right| {
        (
            left.as_of_ms,
            left.entity.to_ascii_lowercase(),
            left.attribute.to_ascii_lowercase(),
            left.value.ordering_key(),
            left.event_id.clone(),
        )
            .cmp(&(
                right.as_of_ms,
                right.entity.to_ascii_lowercase(),
                right.attribute.to_ascii_lowercase(),
                right.value.ordering_key(),
                right.event_id.clone(),
            ))
    });
    events
}

pub fn structured_to_temporal_value(value: &StructuredValue) -> TemporalValue {
    match value {
        StructuredValue::String { value } => TemporalValue::String {
            value: value.clone(),
        },
        StructuredValue::Int { value } => TemporalValue::Int { value: *value },
        StructuredValue::Float { value } => {
            let decimal = Decimal::try_from(*value).unwrap_or(Decimal::ZERO);
            TemporalValue::Decimal {
                value: decimal_to_string(round_decimal(decimal)),
            }
        }
        StructuredValue::Bool { value } => TemporalValue::Bool { value: *value },
        StructuredValue::Date { value } => TemporalValue::Date {
            value: value.clone(),
        },
        StructuredValue::Currency {
            amount,
            currency_code,
        } => {
            let decimal = Decimal::try_from(*amount).unwrap_or(Decimal::ZERO);
            TemporalValue::Currency {
                amount: decimal_to_string(round_decimal(decimal)),
                currency_code: currency_code.clone(),
            }
        }
        StructuredValue::Percent { value } => {
            let decimal = Decimal::try_from(*value).unwrap_or(Decimal::ZERO);
            TemporalValue::Percent {
                value: decimal_to_string(round_decimal(decimal)),
            }
        }
    }
}

fn extract_source_timestamps(evidence_packet: &Value) -> BTreeMap<String, i64> {
    let mut out = BTreeMap::new();
    let sources = evidence_packet
        .get("sources")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();

    for source in sources {
        let url = source
            .get("url")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|entry| !entry.is_empty());
        let Some(url) = url else {
            continue;
        };

        let timestamp = source
            .get("published_at")
            .and_then(parse_timestamp_value)
            .or_else(|| source.get("retrieved_at_ms").and_then(parse_timestamp_value));
        if let Some(timestamp) = timestamp {
            out.insert(url.to_string(), timestamp);
        }
    }

    out
}

fn parse_timestamp_value(value: &Value) -> Option<i64> {
    value
        .as_i64()
        .or_else(|| value.as_u64().and_then(|raw| i64::try_from(raw).ok()))
        .or_else(|| value.as_str().and_then(parse_iso_date_to_unix_ms))
}

fn infer_date_ms_from_row(row: &StructuredRow) -> Option<i64> {
    if row.attribute.to_ascii_lowercase().contains("date") {
        if let StructuredValue::Date { value } = &row.value {
            return parse_iso_date_to_unix_ms(value.as_str());
        }
    }
    None
}

fn parse_iso_date_to_unix_ms(raw: &str) -> Option<i64> {
    let normalized = raw.trim();
    if normalized.len() < 10 {
        return None;
    }
    let date_part = &normalized[0..10];
    let bytes = date_part.as_bytes();
    if bytes.get(4).copied()? != b'-' || bytes.get(7).copied()? != b'-' {
        return None;
    }

    let year = date_part[0..4].parse::<i32>().ok()?;
    let month = date_part[5..7].parse::<u32>().ok()?;
    let day = date_part[8..10].parse::<u32>().ok()?;
    if !(1..=12).contains(&month) || !(1..=31).contains(&day) {
        return None;
    }

    let days_since_epoch = days_from_civil(year, month, day)?;
    Some(days_since_epoch.saturating_mul(86_400_000))
}

fn days_from_civil(year: i32, month: u32, day: u32) -> Option<i64> {
    let y = year - if month <= 2 { 1 } else { 0 };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400;
    let month_i32 = i32::try_from(month).ok()?;
    let day_i32 = i32::try_from(day).ok()?;
    let doy = (153 * (month_i32 + if month_i32 > 2 { -3 } else { 9 }) + 2) / 5 + day_i32 - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    let days = i64::from(era) * 146_097 + i64::from(doe) - 719_468;
    Some(days)
}

fn derive_event_id(
    entity: &str,
    attribute: &str,
    value: &TemporalValue,
    as_of_ms: i64,
    source_refs: &[String],
) -> String {
    let joined_refs = source_refs.join("\x1e");
    let material = format!(
        "entity={}\x1fattribute={}\x1fvalue={}\x1fas_of_ms={}\x1fsource_refs={}",
        entity.to_ascii_lowercase(),
        attribute.to_ascii_lowercase(),
        value.ordering_key(),
        as_of_ms,
        joined_refs
    );
    sha256_hex(material.as_bytes())
}
