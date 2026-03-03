#![forbid(unsafe_code)]

use crate::web_search_plan::structured::schema::{schema_rule_for, StructuredSchemaId};
use crate::web_search_plan::structured::types::{
    StructuredExtraction, StructuredRow, StructuredValue, STRUCTURED_SCHEMA_VERSION,
};
use std::collections::BTreeSet;

pub fn validate_extraction(extraction: &StructuredExtraction) -> Result<(), String> {
    let schema_id = StructuredSchemaId::parse(extraction.schema_id.as_str())
        .ok_or_else(|| format!("unknown structured schema_id {}", extraction.schema_id))?;

    if extraction.query.trim().is_empty() {
        return Err("structured extraction query must not be empty".to_string());
    }
    if extraction.rows.is_empty() {
        return Err("structured extraction rows must not be empty".to_string());
    }

    for (index, row) in extraction.rows.iter().enumerate() {
        validate_row(row).map_err(|error| format!("row {} invalid: {}", index, error))?;
    }

    validate_schema_requirements(schema_id, extraction.rows.as_slice())
}

pub fn validate_row(row: &StructuredRow) -> Result<(), String> {
    if row.entity.trim().is_empty() {
        return Err("entity is required".to_string());
    }
    if row.attribute.trim().is_empty() {
        return Err("attribute is required".to_string());
    }
    if row.source_url.trim().is_empty() {
        return Err("source_url is required".to_string());
    }
    if !row.source_url.starts_with("http://") && !row.source_url.starts_with("https://") {
        return Err("source_url must be http/https".to_string());
    }
    if row.source_ref.trim().is_empty() {
        return Err("source_ref is required".to_string());
    }
    if row.schema_version != STRUCTURED_SCHEMA_VERSION {
        return Err(format!(
            "row schema_version must be {}, got {}",
            STRUCTURED_SCHEMA_VERSION, row.schema_version
        ));
    }

    if let Some(confidence) = row.confidence {
        if !(0.0..=1.0).contains(&confidence) || !confidence.is_finite() {
            return Err("confidence must be finite and in [0,1]".to_string());
        }
    }

    validate_typed_value(&row.value)
}

fn validate_typed_value(value: &StructuredValue) -> Result<(), String> {
    match value {
        StructuredValue::String { value } => {
            if value.trim().is_empty() {
                Err("string value must not be empty".to_string())
            } else {
                Ok(())
            }
        }
        StructuredValue::Int { .. } => Ok(()),
        StructuredValue::Float { value } => {
            if value.is_finite() {
                Ok(())
            } else {
                Err("float value must be finite".to_string())
            }
        }
        StructuredValue::Bool { .. } => Ok(()),
        StructuredValue::Date { value } => {
            if is_date_string(value.as_str()) {
                Ok(())
            } else {
                Err(format!("date value invalid {}", value))
            }
        }
        StructuredValue::Currency {
            amount,
            currency_code,
        } => {
            if !amount.is_finite() {
                return Err("currency amount must be finite".to_string());
            }
            if !is_currency_code(currency_code) {
                return Err(format!("currency code invalid {}", currency_code));
            }
            Ok(())
        }
        StructuredValue::Percent { value } => {
            if value.is_finite() && (0.0..=100.0).contains(value) {
                Ok(())
            } else {
                Err(format!("percent must be in [0,100], got {}", value))
            }
        }
    }
}

fn validate_schema_requirements(
    schema_id: StructuredSchemaId,
    rows: &[StructuredRow],
) -> Result<(), String> {
    let rule = schema_rule_for(schema_id);
    if rule.required_attributes.is_empty() {
        return Ok(());
    }

    let present = rows
        .iter()
        .map(|row| row.attribute.to_ascii_lowercase())
        .collect::<BTreeSet<String>>();

    for attribute in rule.required_attributes {
        if !present.contains(*attribute) {
            return Err(format!(
                "schema {} missing required attribute {}",
                schema_id.as_str(),
                attribute
            ));
        }
    }

    Ok(())
}

fn is_currency_code(code: &str) -> bool {
    code.len() == 3 && code.chars().all(|ch| ch.is_ascii_uppercase())
}

fn is_date_string(value: &str) -> bool {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return false;
    }
    if trimmed.chars().all(|ch| ch.is_ascii_digit()) {
        return trimmed.parse::<i64>().is_ok();
    }

    if trimmed.len() < 10 {
        return false;
    }
    let bytes = trimmed.as_bytes();
    if bytes.get(4) != Some(&b'-') || bytes.get(7) != Some(&b'-') {
        return false;
    }
    let year = &trimmed[0..4];
    let month = &trimmed[5..7];
    let day = &trimmed[8..10];
    year.chars().all(|ch| ch.is_ascii_digit())
        && month.chars().all(|ch| ch.is_ascii_digit())
        && day.chars().all(|ch| ch.is_ascii_digit())
}
