#![forbid(unsafe_code)]

use crate::web_search_plan::document::{DocumentError, DocumentErrorKind};
use crate::web_search_plan::structured::types::{
    StructuredRow, StructuredValue, STRUCTURED_SCHEMA_VERSION,
};

pub fn extract_table_rows(
    normalized_text: &str,
    source_url: &str,
    schema_id: &str,
    max_rows: usize,
) -> Result<Vec<StructuredRow>, DocumentError> {
    let lines = normalized_text
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<&str>>();

    let mut table_start = None;
    for (index, line) in lines.iter().enumerate() {
        if line.matches('|').count() >= 2 {
            table_start = Some(index);
            break;
        }
    }

    let Some(start_index) = table_start else {
        return Ok(Vec::new());
    };

    let headers = split_pipe_line(lines[start_index]);
    if headers.len() < 2 {
        return Err(DocumentError::new(
            "document_pdf_tables",
            DocumentErrorKind::PolicyViolation,
            None,
            "table header ambiguous in PDF text",
            0,
        ));
    }

    let mut rows = Vec::new();
    for (line_index, line) in lines.iter().enumerate().skip(start_index + 1) {
        if rows.len() >= max_rows {
            break;
        }
        if line.matches('|').count() < headers.len().saturating_sub(1) {
            break;
        }

        let cells = split_pipe_line(line);
        if cells.len() != headers.len() {
            return Err(DocumentError::new(
                "document_pdf_tables",
                DocumentErrorKind::PolicyViolation,
                None,
                "table row width does not match header width",
                0,
            ));
        }

        let entity = sanitize_cell(cells[0].as_str());
        if entity.is_empty() {
            continue;
        }

        for (column_index, header) in headers.iter().enumerate().skip(1) {
            let attribute = header_to_attribute(header);
            let cell = sanitize_cell(cells[column_index].as_str());
            if cell.is_empty() {
                continue;
            }

            let value = parse_typed_cell(cell.as_str());
            let row = StructuredRow {
                entity: entity.clone(),
                attribute,
                value,
                unit: None,
                as_of_ms: None,
                source_url: source_url.to_string(),
                source_ref: format!("{}#table_row_{}", source_url, line_index),
                confidence: None,
                schema_version: STRUCTURED_SCHEMA_VERSION.to_string(),
            };
            rows.push(row);
        }
    }

    if rows.is_empty() && schema_id == "pdf_table_v1" {
        return Err(DocumentError::new(
            "document_pdf_tables",
            DocumentErrorKind::InsufficientEvidence,
            None,
            "pdf table parsing found no structured rows",
            0,
        ));
    }

    Ok(rows)
}

fn split_pipe_line(line: &str) -> Vec<String> {
    line.split('|').map(sanitize_cell).collect::<Vec<String>>()
}

fn sanitize_cell(raw: &str) -> String {
    raw.split_whitespace()
        .collect::<Vec<&str>>()
        .join(" ")
        .trim()
        .to_string()
}

fn header_to_attribute(header: &str) -> String {
    header
        .trim()
        .to_ascii_lowercase()
        .replace(' ', "_")
        .replace('-', "_")
}

fn parse_typed_cell(cell: &str) -> StructuredValue {
    if let Some(percent) = parse_percent(cell) {
        return StructuredValue::Percent { value: percent };
    }

    if let Some((amount, currency)) = parse_currency(cell) {
        return StructuredValue::Currency {
            amount,
            currency_code: currency,
        };
    }

    if let Some(value) = parse_bool(cell) {
        return StructuredValue::Bool { value };
    }

    if is_date_like(cell) {
        return StructuredValue::Date {
            value: cell.to_string(),
        };
    }

    if let Ok(value) = cell.parse::<i64>() {
        return StructuredValue::Int { value };
    }

    if let Ok(value) = cell.parse::<f64>() {
        if value.is_finite() {
            return StructuredValue::Float { value };
        }
    }

    StructuredValue::String {
        value: cell.to_string(),
    }
}

fn parse_percent(cell: &str) -> Option<f64> {
    let value = cell.strip_suffix('%')?.trim().parse::<f64>().ok()?;
    if value.is_finite() && (0.0..=100.0).contains(&value) {
        Some(value)
    } else {
        None
    }
}

fn parse_currency(cell: &str) -> Option<(f64, String)> {
    if let Some(amount) = cell
        .strip_prefix('$')
        .and_then(|raw| raw.replace(',', "").parse::<f64>().ok())
    {
        if amount.is_finite() {
            return Some((amount, "USD".to_string()));
        }
    }

    let parts = cell.split_whitespace().collect::<Vec<&str>>();
    if parts.len() == 2
        && parts[1].len() == 3
        && parts[1].chars().all(|ch| ch.is_ascii_alphabetic())
    {
        if let Ok(amount) = parts[0].replace(',', "").parse::<f64>() {
            if amount.is_finite() {
                return Some((amount, parts[1].to_ascii_uppercase()));
            }
        }
    }

    None
}

fn parse_bool(cell: &str) -> Option<bool> {
    match cell.to_ascii_lowercase().as_str() {
        "true" | "yes" | "present" => Some(true),
        "false" | "no" | "absent" => Some(false),
        _ => None,
    }
}

fn is_date_like(cell: &str) -> bool {
    let bytes = cell.as_bytes();
    if bytes.len() >= 10
        && bytes.get(4) == Some(&b'-')
        && bytes.get(7) == Some(&b'-')
        && cell[0..4].chars().all(|ch| ch.is_ascii_digit())
        && cell[5..7].chars().all(|ch| ch.is_ascii_digit())
        && cell[8..10].chars().all(|ch| ch.is_ascii_digit())
    {
        return true;
    }
    cell.chars().all(|ch| ch.is_ascii_digit()) && cell.parse::<i64>().is_ok()
}
