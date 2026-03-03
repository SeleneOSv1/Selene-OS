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
    if schema_id != "filing_sec_like_v1" {
        return Err(DocumentError::new(
            "document_filing_pack",
            DocumentErrorKind::PolicyViolation,
            None,
            format!("sec_like parser received unsupported schema {}", schema_id),
            0,
        ));
    }

    let mut company_name: Option<String> = None;
    let mut period_end_date: Option<String> = None;
    let mut revenue: Option<(f64, String)> = None;
    let mut net_income: Option<(f64, String)> = None;
    let mut risk_present: Option<bool> = None;

    for line in normalized_text.lines() {
        let lower = line.to_ascii_lowercase();
        if company_name.is_none() && lower.contains("form 10-k") {
            company_name = Some(
                line.replace("FORM 10-K", "")
                    .replace("Form 10-K", "")
                    .trim()
                    .to_string(),
            );
            continue;
        }
        if period_end_date.is_none() && lower.contains("period end date:") {
            if let Some((_, rhs)) = line.split_once(':') {
                let value = rhs.trim();
                if !value.is_empty() {
                    period_end_date = Some(value.to_string());
                }
            }
            continue;
        }
        if revenue.is_none() && lower.starts_with("revenue:") {
            if let Some((_, rhs)) = line.split_once(':') {
                revenue = parse_currency(rhs.trim());
            }
            continue;
        }
        if net_income.is_none() && lower.starts_with("net income:") {
            if let Some((_, rhs)) = line.split_once(':') {
                net_income = parse_currency(rhs.trim());
            }
            continue;
        }
        if risk_present.is_none() && lower.contains("risk factors") {
            risk_present = Some(!lower.contains("not present"));
        }
    }

    let entity = company_name.unwrap_or_else(|| "unknown_company".to_string());
    let mut rows = vec![StructuredRow {
        entity: entity.clone(),
        attribute: "company_name".to_string(),
        value: StructuredValue::String {
            value: entity.clone(),
        },
        unit: None,
        as_of_ms: None,
        source_url: source_url.to_string(),
        source_ref: source_url.to_string(),
        confidence: None,
        schema_version: STRUCTURED_SCHEMA_VERSION.to_string(),
    }];

    if let Some(value) = period_end_date {
        rows.push(StructuredRow {
            entity: entity.clone(),
            attribute: "period_end_date".to_string(),
            value: StructuredValue::Date { value },
            unit: None,
            as_of_ms: None,
            source_url: source_url.to_string(),
            source_ref: source_url.to_string(),
            confidence: None,
            schema_version: STRUCTURED_SCHEMA_VERSION.to_string(),
        });
    }

    if let Some((amount, currency_code)) = revenue {
        rows.push(StructuredRow {
            entity: entity.clone(),
            attribute: "revenue".to_string(),
            value: StructuredValue::Currency {
                amount,
                currency_code,
            },
            unit: None,
            as_of_ms: None,
            source_url: source_url.to_string(),
            source_ref: source_url.to_string(),
            confidence: None,
            schema_version: STRUCTURED_SCHEMA_VERSION.to_string(),
        });
    }

    if let Some((amount, currency_code)) = net_income {
        rows.push(StructuredRow {
            entity: entity.clone(),
            attribute: "net_income".to_string(),
            value: StructuredValue::Currency {
                amount,
                currency_code,
            },
            unit: None,
            as_of_ms: None,
            source_url: source_url.to_string(),
            source_ref: source_url.to_string(),
            confidence: None,
            schema_version: STRUCTURED_SCHEMA_VERSION.to_string(),
        });
    }

    if let Some(value) = risk_present {
        rows.push(StructuredRow {
            entity,
            attribute: "risk_factors_section_present".to_string(),
            value: StructuredValue::Bool { value },
            unit: None,
            as_of_ms: None,
            source_url: source_url.to_string(),
            source_ref: source_url.to_string(),
            confidence: None,
            schema_version: STRUCTURED_SCHEMA_VERSION.to_string(),
        });
    }

    if rows.is_empty() {
        return Err(DocumentError::new(
            "document_filing_pack",
            DocumentErrorKind::InsufficientEvidence,
            None,
            "sec_like parser produced no rows",
            0,
        ));
    }

    Ok(rows)
}

fn parse_currency(raw: &str) -> Option<(f64, String)> {
    let trimmed = raw.trim();
    if let Some(value) = trimmed
        .strip_prefix('$')
        .and_then(|candidate| candidate.replace(',', "").parse::<f64>().ok())
    {
        if value.is_finite() {
            return Some((value, "USD".to_string()));
        }
    }

    let parts = trimmed.split_whitespace().collect::<Vec<&str>>();
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
