#![forbid(unsafe_code)]

use serde_json::Value;

pub const TABLE_RENDER_VERSION: &str = "1.0.0";

pub fn render_competitive_pricing_table(packet: &Value) -> Result<String, String> {
    let rows = packet
        .get("pricing_table")
        .and_then(Value::as_array)
        .ok_or_else(|| "comparison packet missing pricing_table".to_string())?;

    let mut normalized_rows = rows
        .iter()
        .map(|row| {
            vec![
                get_string(row, "entity_id"),
                get_string(row, "price_value"),
                get_string(row, "currency"),
                get_string(row, "billing_period"),
                get_string(row, "tax_included"),
            ]
        })
        .collect::<Vec<Vec<String>>>();
    normalized_rows.sort();

    Ok(render_text_table(
        &["entity_id", "price", "currency", "billing_period", "tax_included"],
        normalized_rows,
    ))
}

pub fn render_temporal_changes_table(packet: &Value) -> Result<String, String> {
    let rows = packet
        .get("changes")
        .and_then(Value::as_array)
        .ok_or_else(|| "temporal packet missing changes".to_string())?;

    let mut normalized_rows = rows
        .iter()
        .map(|row| {
            vec![
                get_string(row, "key"),
                nested_value_to_string(row, "prior_value"),
                nested_value_to_string(row, "new_value"),
                get_string(row, "change_type"),
                get_string(row, "delta_value"),
            ]
        })
        .collect::<Vec<Vec<String>>>();
    normalized_rows.sort();

    Ok(render_text_table(
        &["key", "prior", "new", "change_type", "delta"],
        normalized_rows,
    ))
}

pub fn render_risk_factor_table(packet: &Value) -> Result<String, String> {
    let rows = packet
        .get("factor_breakdown")
        .and_then(Value::as_array)
        .ok_or_else(|| "risk packet missing factor_breakdown".to_string())?;

    let mut normalized_rows = rows
        .iter()
        .map(|row| {
            vec![
                get_string(row, "factor_id"),
                get_string(row, "score"),
                get_string(row, "weight"),
                join_refs(row.get("evidence_refs")),
            ]
        })
        .collect::<Vec<Vec<String>>>();
    normalized_rows.sort();

    Ok(render_text_table(
        &["factor_id", "score", "weight", "evidence_refs"],
        normalized_rows,
    ))
}

pub fn render_text_table(headers: &[&str], rows: Vec<Vec<String>>) -> String {
    let mut widths = headers.iter().map(|header| header.len()).collect::<Vec<usize>>();
    for row in &rows {
        for (index, cell) in row.iter().enumerate() {
            if let Some(width) = widths.get_mut(index) {
                *width = (*width).max(cell.len());
            }
        }
    }

    let mut lines = Vec::new();
    lines.push(render_row(
        headers.iter().map(|entry| entry.to_string()).collect(),
        widths.as_slice(),
    ));
    lines.push(render_separator(widths.as_slice()));
    for row in rows {
        lines.push(render_row(row, widths.as_slice()));
    }
    lines.join("\n")
}

fn render_row(cells: Vec<String>, widths: &[usize]) -> String {
    let mut out = String::new();
    out.push('|');
    for (index, cell) in cells.iter().enumerate() {
        let width = widths.get(index).copied().unwrap_or(cell.len());
        out.push(' ');
        out.push_str(cell);
        if width > cell.len() {
            out.push_str(&" ".repeat(width - cell.len()));
        }
        out.push(' ');
        out.push('|');
    }
    out
}

fn render_separator(widths: &[usize]) -> String {
    let mut out = String::new();
    out.push('|');
    for width in widths {
        out.push(' ');
        out.push_str(&"-".repeat(*width));
        out.push(' ');
        out.push('|');
    }
    out
}

fn get_string(row: &Value, field: &str) -> String {
    row.get(field)
        .map(value_to_string)
        .unwrap_or_else(|| "unknown".to_string())
}

fn nested_value_to_string(row: &Value, field: &str) -> String {
    let Some(value) = row.get(field) else {
        return "unknown".to_string();
    };
    if value.is_null() {
        return "unknown".to_string();
    }
    value_to_string(value)
}

fn join_refs(value: Option<&Value>) -> String {
    let Some(raw) = value.and_then(Value::as_array) else {
        return "unknown".to_string();
    };
    let mut refs = raw
        .iter()
        .filter_map(Value::as_str)
        .map(ToString::to_string)
        .collect::<Vec<String>>();
    refs.sort();
    refs.dedup();
    refs.join(",")
}

fn value_to_string(value: &Value) -> String {
    match value {
        Value::Null => "unknown".to_string(),
        Value::String(raw) => raw.to_string(),
        Value::Bool(raw) => raw.to_string(),
        Value::Number(raw) => raw.to_string(),
        Value::Object(map) => {
            if let Some(entry) = map.get("value") {
                return value_to_string(entry);
            }
            if let Some(entry) = map.get("kind").and_then(Value::as_str) {
                return entry.to_string();
            }
            serde_json::to_string(map).unwrap_or_else(|_| "unknown".to_string())
        }
        Value::Array(entries) => entries
            .iter()
            .map(value_to_string)
            .collect::<Vec<String>>()
            .join(","),
    }
}
