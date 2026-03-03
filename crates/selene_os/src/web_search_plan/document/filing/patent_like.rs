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
    if schema_id != "filing_patent_like_v1" {
        return Err(DocumentError::new(
            "document_filing_pack",
            DocumentErrorKind::PolicyViolation,
            None,
            format!(
                "patent_like parser received unsupported schema {}",
                schema_id
            ),
            0,
        ));
    }

    let mut patent_number: Option<String> = None;
    let mut filing_date: Option<String> = None;
    let mut assignee: Option<String> = None;
    let mut inventors = Vec::new();

    for line in normalized_text.lines() {
        let lower = line.to_ascii_lowercase();
        if patent_number.is_none() && lower.starts_with("patent number:") {
            if let Some((_, rhs)) = line.split_once(':') {
                let value = rhs.trim();
                if !value.is_empty() {
                    patent_number = Some(value.to_string());
                }
            }
            continue;
        }
        if filing_date.is_none() && lower.starts_with("filing date:") {
            if let Some((_, rhs)) = line.split_once(':') {
                let value = rhs.trim();
                if !value.is_empty() {
                    filing_date = Some(value.to_string());
                }
            }
            continue;
        }
        if assignee.is_none() && lower.starts_with("assignee:") {
            if let Some((_, rhs)) = line.split_once(':') {
                let value = rhs.trim();
                if !value.is_empty() {
                    assignee = Some(value.to_string());
                }
            }
            continue;
        }
        if inventors.is_empty() && lower.starts_with("inventors:") {
            if let Some((_, rhs)) = line.split_once(':') {
                for inventor in rhs.split(';') {
                    let cleaned = inventor.trim();
                    if !cleaned.is_empty() {
                        inventors.push(cleaned.to_string());
                    }
                }
            }
        }
    }

    let entity = patent_number
        .clone()
        .unwrap_or_else(|| "unknown_patent".to_string());
    let mut rows = Vec::new();

    if let Some(value) = patent_number {
        rows.push(StructuredRow {
            entity: entity.clone(),
            attribute: "patent_number".to_string(),
            value: StructuredValue::String { value },
            unit: None,
            as_of_ms: None,
            source_url: source_url.to_string(),
            source_ref: source_url.to_string(),
            confidence: None,
            schema_version: STRUCTURED_SCHEMA_VERSION.to_string(),
        });
    }

    if let Some(value) = filing_date {
        rows.push(StructuredRow {
            entity: entity.clone(),
            attribute: "filing_date".to_string(),
            value: StructuredValue::Date { value },
            unit: None,
            as_of_ms: None,
            source_url: source_url.to_string(),
            source_ref: source_url.to_string(),
            confidence: None,
            schema_version: STRUCTURED_SCHEMA_VERSION.to_string(),
        });
    }

    for inventor in inventors {
        rows.push(StructuredRow {
            entity: entity.clone(),
            attribute: "inventor".to_string(),
            value: StructuredValue::String { value: inventor },
            unit: None,
            as_of_ms: None,
            source_url: source_url.to_string(),
            source_ref: source_url.to_string(),
            confidence: None,
            schema_version: STRUCTURED_SCHEMA_VERSION.to_string(),
        });
    }

    if let Some(value) = assignee {
        rows.push(StructuredRow {
            entity,
            attribute: "assignee".to_string(),
            value: StructuredValue::String { value },
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
            "patent_like parser produced no fields",
            0,
        ));
    }

    Ok(rows)
}
