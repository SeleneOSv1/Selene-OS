#![forbid(unsafe_code)]

use crate::web_search_plan::structured::types::{StructuredRow, StructuredValue};

pub fn normalize_text(input: &str) -> String {
    input
        .split_whitespace()
        .collect::<Vec<&str>>()
        .join(" ")
        .trim()
        .to_string()
}

pub fn normalize_row(mut row: StructuredRow) -> StructuredRow {
    row.entity = normalize_text(&row.entity);
    row.attribute = normalize_text(&row.attribute);
    row.source_url = row.source_url.trim().to_ascii_lowercase();
    row.source_ref = row.source_ref.trim().to_string();
    row.unit = row.unit.map(|value| normalize_text(&value));
    row.value = normalize_value(row.value);
    row
}

pub fn normalize_value(value: StructuredValue) -> StructuredValue {
    match value {
        StructuredValue::String { value } => StructuredValue::String {
            value: normalize_text(&value),
        },
        StructuredValue::Date { value } => StructuredValue::Date {
            value: value.trim().to_string(),
        },
        StructuredValue::Currency {
            amount,
            currency_code,
        } => StructuredValue::Currency {
            amount,
            currency_code: currency_code.trim().to_ascii_uppercase(),
        },
        other => other,
    }
}

pub fn sort_rows_deterministically(rows: &mut Vec<StructuredRow>) {
    rows.sort_by(|left, right| row_ordering_key(left).cmp(&row_ordering_key(right)));
}

pub fn row_ordering_key(row: &StructuredRow) -> (String, String, String, String) {
    (
        row.entity.to_ascii_lowercase(),
        row.attribute.to_ascii_lowercase(),
        row.source_url.to_ascii_lowercase(),
        row.value.ordering_key(),
    )
}
