#![forbid(unsafe_code)]

use crate::web_search_plan::document::{DocumentError, DocumentErrorKind};
use crate::web_search_plan::structured::types::{
    StructuredRow, StructuredValue, STRUCTURED_SCHEMA_VERSION,
};

pub fn parse_rows(
    normalized_text: &str,
    source_url: &str,
    schema_id: &str,
) -> Result<Vec<StructuredRow>, DocumentError> {
    if schema_id != "filing_financials_like_v1" {
        return Err(DocumentError::new(
            "document_filing_pack",
            DocumentErrorKind::PolicyViolation,
            None,
            format!(
                "financials_like parser received unsupported schema {}",
                schema_id
            ),
            0,
        ));
    }

    let mut rows = Vec::new();
    for line in normalized_text.lines() {
        let lower = line.to_ascii_lowercase();
        if lower.contains("income statement") && lower.contains("revenue") {
            if let Some((amount, currency_code)) = trailing_currency(line) {
                rows.push(make_row(
                    "IncomeStatement",
                    "revenue",
                    StructuredValue::Currency {
                        amount,
                        currency_code,
                    },
                    source_url,
                ));
            }
        }
        if lower.contains("balance sheet") && lower.contains("total assets") {
            if let Some((amount, currency_code)) = trailing_currency(line) {
                rows.push(make_row(
                    "BalanceSheet",
                    "total_assets",
                    StructuredValue::Currency {
                        amount,
                        currency_code,
                    },
                    source_url,
                ));
            }
        }
        if lower.contains("cash flow") && lower.contains("operating cash flow") {
            if let Some((amount, currency_code)) = trailing_currency(line) {
                rows.push(make_row(
                    "CashFlow",
                    "operating_cash_flow",
                    StructuredValue::Currency {
                        amount,
                        currency_code,
                    },
                    source_url,
                ));
            }
        }
    }

    if rows.is_empty() {
        return Err(DocumentError::new(
            "document_filing_pack",
            DocumentErrorKind::InsufficientEvidence,
            None,
            "financials_like parser found no deterministic statements",
            0,
        ));
    }

    Ok(rows)
}

fn make_row(
    entity: &str,
    attribute: &str,
    value: StructuredValue,
    source_url: &str,
) -> StructuredRow {
    StructuredRow {
        entity: entity.to_string(),
        attribute: attribute.to_string(),
        value,
        unit: None,
        as_of_ms: None,
        source_url: source_url.to_string(),
        source_ref: source_url.to_string(),
        confidence: None,
        schema_version: STRUCTURED_SCHEMA_VERSION.to_string(),
    }
}

fn trailing_currency(line: &str) -> Option<(f64, String)> {
    let trailing = line.split(':').next_back()?.trim();
    if let Some(value) = trailing
        .strip_prefix('$')
        .and_then(|candidate| candidate.replace(',', "").parse::<f64>().ok())
    {
        if value.is_finite() {
            return Some((value, "USD".to_string()));
        }
    }

    let parts = trailing.split_whitespace().collect::<Vec<&str>>();
    if parts.len() == 2
        && parts[1].len() == 3
        && parts[1].chars().all(|ch| ch.is_ascii_alphabetic())
    {
        if let Ok(value) = parts[0].replace(',', "").parse::<f64>() {
            if value.is_finite() {
                return Some((value, parts[1].to_ascii_uppercase()));
            }
        }
    }

    None
}
