#![forbid(unsafe_code)]

use crate::web_search_plan::structured::types::StructuredRow;
use serde_json::Value;
use std::collections::BTreeSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DocumentSchemaKind {
    PdfTableV1,
    FilingSecLikeV1,
    FilingFinancialsLikeV1,
    FilingPatentLikeV1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SchemaDescriptor {
    pub schema_id: &'static str,
    pub required_attributes: &'static [&'static str],
    pub partial_allowed: bool,
}

pub fn schema_descriptor(kind: DocumentSchemaKind) -> SchemaDescriptor {
    match kind {
        DocumentSchemaKind::PdfTableV1 => SchemaDescriptor {
            schema_id: "pdf_table_v1",
            required_attributes: &["revenue", "margin", "as_of"],
            partial_allowed: false,
        },
        DocumentSchemaKind::FilingSecLikeV1 => SchemaDescriptor {
            schema_id: "filing_sec_like_v1",
            required_attributes: &[
                "company_name",
                "period_end_date",
                "revenue",
                "net_income",
                "risk_factors_section_present",
            ],
            partial_allowed: false,
        },
        DocumentSchemaKind::FilingFinancialsLikeV1 => SchemaDescriptor {
            schema_id: "filing_financials_like_v1",
            required_attributes: &["revenue", "total_assets", "operating_cash_flow"],
            partial_allowed: false,
        },
        DocumentSchemaKind::FilingPatentLikeV1 => SchemaDescriptor {
            schema_id: "filing_patent_like_v1",
            required_attributes: &["patent_number", "filing_date", "inventor", "assignee"],
            partial_allowed: false,
        },
    }
}

pub fn schema_kind_from_tool_request(tool_request_packet: &Value) -> DocumentSchemaKind {
    if let Some(kind) = tool_request_packet
        .get("budgets")
        .and_then(Value::as_object)
        .and_then(|budgets| budgets.get("document_schema"))
        .and_then(Value::as_str)
        .and_then(parse_kind)
    {
        return kind;
    }

    let query = tool_request_packet
        .get("query")
        .and_then(Value::as_str)
        .unwrap_or_default();
    parse_kind(query).unwrap_or(DocumentSchemaKind::FilingSecLikeV1)
}

pub fn validate_required_rows(
    kind: DocumentSchemaKind,
    rows: &[StructuredRow],
) -> Result<(), String> {
    let descriptor = schema_descriptor(kind);
    if descriptor.required_attributes.is_empty() || descriptor.partial_allowed {
        return Ok(());
    }

    let present = rows
        .iter()
        .map(|row| row.attribute.to_ascii_lowercase())
        .collect::<BTreeSet<String>>();

    let missing = descriptor
        .required_attributes
        .iter()
        .filter(|required| !present.contains(**required))
        .copied()
        .collect::<Vec<&str>>();
    if missing.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "schema {} missing required attributes: {}",
            descriptor.schema_id,
            missing.join(",")
        ))
    }
}

fn parse_kind(raw: &str) -> Option<DocumentSchemaKind> {
    let normalized = raw.trim().to_ascii_lowercase();
    if normalized.contains("pdf_table_v1") || normalized.contains("table") {
        return Some(DocumentSchemaKind::PdfTableV1);
    }
    if normalized.contains("filing_financials_like_v1") || normalized.contains("financials_like") {
        return Some(DocumentSchemaKind::FilingFinancialsLikeV1);
    }
    if normalized.contains("filing_patent_like_v1") || normalized.contains("patent_like") {
        return Some(DocumentSchemaKind::FilingPatentLikeV1);
    }
    if normalized.contains("filing_sec_like_v1") || normalized.contains("sec_like") {
        return Some(DocumentSchemaKind::FilingSecLikeV1);
    }
    None
}
