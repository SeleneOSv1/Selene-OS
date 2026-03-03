#![forbid(unsafe_code)]

pub mod filing;
pub mod ocr;
pub mod pdf_fetch;
pub mod pdf_tables;
pub mod pdf_text;
pub mod schema_map;

use crate::web_search_plan::proxy::proxy_config::{ProxyConfig, SystemEnvProvider};
use crate::web_search_plan::proxy::ProxyMode;
use crate::web_search_plan::structured::normalize::sort_rows_deterministically;
use crate::web_search_plan::structured::types::{
    StructuredExtraction, STRUCTURED_ENGINE_ID, STRUCTURED_SCHEMA_VERSION,
};
use crate::web_search_plan::structured::validator::{validate_extraction, validate_row};
use crate::web_search_plan::{
    document::schema_map::DocumentSchemaKind, structured::types::StructuredRow,
};
use serde_json::{json, Value};

pub const DOCUMENT_ENGINE_ID: &str = STRUCTURED_ENGINE_ID;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DocumentErrorKind {
    ProviderUnconfigured,
    ProviderUpstreamFailed,
    TimeoutExceeded,
    EmptyResults,
    PolicyViolation,
    InsufficientEvidence,
    InvalidInput,
}

impl DocumentErrorKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ProviderUnconfigured => "provider_unconfigured",
            Self::ProviderUpstreamFailed => "provider_upstream_failed",
            Self::TimeoutExceeded => "timeout_exceeded",
            Self::EmptyResults => "empty_results",
            Self::PolicyViolation => "policy_violation",
            Self::InsufficientEvidence => "insufficient_evidence",
            Self::InvalidInput => "invalid_input",
        }
    }

    pub const fn reason_code(self) -> &'static str {
        match self {
            Self::ProviderUnconfigured => "provider_unconfigured",
            Self::ProviderUpstreamFailed | Self::InvalidInput => "provider_upstream_failed",
            Self::TimeoutExceeded => "timeout_exceeded",
            Self::EmptyResults => "empty_results",
            Self::PolicyViolation => "policy_violation",
            Self::InsufficientEvidence => "insufficient_evidence",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocumentError {
    pub stage: String,
    pub kind: DocumentErrorKind,
    pub status_code: Option<u16>,
    pub message: String,
    pub latency_ms: u64,
}

impl DocumentError {
    pub fn new(
        stage: impl Into<String>,
        kind: DocumentErrorKind,
        status_code: Option<u16>,
        message: impl Into<String>,
        latency_ms: u64,
    ) -> Self {
        Self {
            stage: stage.into(),
            kind,
            status_code,
            message: message.into(),
            latency_ms,
        }
    }

    pub const fn reason_code(&self) -> &'static str {
        self.kind.reason_code()
    }
}

#[derive(Debug, Clone)]
pub struct DocumentRuntimeConfig {
    pub max_pdf_bytes: usize,
    pub max_extracted_chars: usize,
    pub pdf_fetch_timeout_ms: u64,
    pub table_row_limit: usize,
    pub ocr_options: ocr::OcrOptions,
    pub proxy_config: ProxyConfig,
}

impl Default for DocumentRuntimeConfig {
    fn default() -> Self {
        let env = SystemEnvProvider;
        let proxy_mode_raw =
            std::env::var("SELENE_DOCUMENT_PROXY_MODE").unwrap_or_else(|_| "off".to_string());
        let proxy_mode = ProxyMode::parse(&proxy_mode_raw).unwrap_or(ProxyMode::Off);

        Self {
            max_pdf_bytes: 8 * 1024 * 1024,
            max_extracted_chars: 180_000,
            pdf_fetch_timeout_ms: 4_000,
            table_row_limit: 256,
            ocr_options: ocr::OcrOptions::from_env(),
            proxy_config: ProxyConfig::from_env(proxy_mode, &env),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DocumentPipelineResult {
    pub extraction: StructuredExtraction,
    pub evidence_packet: Value,
}

#[derive(Debug, Clone)]
pub struct ParsedDocumentRequest {
    pub trace_id: String,
    pub query: String,
    pub pdf_url: String,
    pub created_at_ms: i64,
    pub now_ms: i64,
    pub intended_consumers: Vec<String>,
    pub importance_tier: String,
    pub schema_kind: DocumentSchemaKind,
}

pub fn execute_document_pipeline_from_tool_request(
    tool_request_packet: &Value,
    now_ms: i64,
    config: &DocumentRuntimeConfig,
) -> Result<DocumentPipelineResult, DocumentError> {
    let request = parse_document_request(tool_request_packet, now_ms)?;

    let mut provider_runs = Vec::new();
    let mut sources = Vec::new();

    let fetched = pdf_fetch::fetch_pdf(&request, config)?;
    provider_runs.push(json!({
        "provider_id": "document_pdf_fetch",
        "endpoint": "document_pdf_fetch",
        "latency_ms": fetched.latency_ms,
        "results_count": 1,
        "error": Value::Null,
    }));
    sources.push(json!({
        "title": "Document PDF",
        "url": fetched.final_url,
        "snippet": "pdf source fetched for deterministic document parsing",
        "media_type": "document",
        "provider_id": "document_pdf_fetch",
        "rank": 1,
        "canonical_url": fetched.final_url.to_ascii_lowercase(),
    }));

    let mut text_output = pdf_text::extract_text_from_pdf(
        fetched.bytes.as_slice(),
        config.max_extracted_chars,
        request.now_ms,
    )?;
    provider_runs.push(json!({
        "provider_id": "document_pdf_text",
        "endpoint": "document_pdf_text",
        "latency_ms": text_output.latency_ms,
        "results_count": text_output.page_count,
        "error": Value::Null,
    }));

    if text_output.extracted_text.trim().is_empty() {
        let ocr_output = ocr::extract_text_from_pdf_with_ocr(
            fetched.bytes.as_slice(),
            &config.ocr_options,
            config.max_extracted_chars,
        )?;
        provider_runs.push(json!({
            "provider_id": "document_ocr",
            "endpoint": "document_ocr",
            "latency_ms": ocr_output.latency_ms,
            "results_count": ocr_output.pages_processed,
            "error": Value::Null,
        }));
        text_output.extracted_text = ocr_output.extracted_text;
    }

    let schema_descriptor = schema_map::schema_descriptor(request.schema_kind);
    let mut rows: Vec<StructuredRow> = match request.schema_kind {
        DocumentSchemaKind::PdfTableV1 => {
            let table_rows = pdf_tables::extract_table_rows(
                text_output.extracted_text.as_str(),
                fetched.final_url.as_str(),
                schema_descriptor.schema_id,
                config.table_row_limit,
            )?;
            provider_runs.push(json!({
                "provider_id": "document_pdf_tables",
                "endpoint": "document_pdf_tables",
                "latency_ms": 0,
                "results_count": table_rows.len(),
                "error": Value::Null,
            }));
            table_rows
        }
        DocumentSchemaKind::FilingSecLikeV1 => {
            let parsed = filing::sec_like::parse_rows(
                text_output.extracted_text.as_str(),
                fetched.final_url.as_str(),
                schema_descriptor.schema_id,
            )?;
            provider_runs.push(json!({
                "provider_id": "document_filing_pack",
                "endpoint": "document_filing_pack",
                "latency_ms": 0,
                "results_count": parsed.len(),
                "error": Value::Null,
            }));
            parsed
        }
        DocumentSchemaKind::FilingFinancialsLikeV1 => {
            let parsed = filing::financials_like::parse_rows(
                text_output.extracted_text.as_str(),
                fetched.final_url.as_str(),
                schema_descriptor.schema_id,
            )?;
            provider_runs.push(json!({
                "provider_id": "document_filing_pack",
                "endpoint": "document_filing_pack",
                "latency_ms": 0,
                "results_count": parsed.len(),
                "error": Value::Null,
            }));
            parsed
        }
        DocumentSchemaKind::FilingPatentLikeV1 => {
            let parsed = filing::patent_like::parse_rows(
                text_output.extracted_text.as_str(),
                fetched.final_url.as_str(),
                schema_descriptor.schema_id,
            )?;
            provider_runs.push(json!({
                "provider_id": "document_filing_pack",
                "endpoint": "document_filing_pack",
                "latency_ms": 0,
                "results_count": parsed.len(),
                "error": Value::Null,
            }));
            parsed
        }
    };

    for row in &rows {
        validate_row(row).map_err(|error| {
            DocumentError::new(
                "document_schema_validation",
                DocumentErrorKind::PolicyViolation,
                None,
                format!("structured row validation failed: {}", error),
                0,
            )
        })?;
    }
    schema_map::validate_required_rows(request.schema_kind, rows.as_slice()).map_err(|error| {
        DocumentError::new(
            "document_required_fields",
            DocumentErrorKind::InsufficientEvidence,
            None,
            error,
            0,
        )
    })?;

    sort_rows_deterministically(&mut rows);

    let extraction = StructuredExtraction {
        query: request.query.clone(),
        rows,
        schema_id: schema_descriptor.schema_id.to_string(),
        extracted_at_ms: request.now_ms,
        provider_runs: provider_runs.clone(),
        sources: sources.clone(),
        errors: Vec::new(),
    };
    validate_extraction(&extraction).map_err(|error| {
        DocumentError::new(
            "document_schema_validation",
            DocumentErrorKind::PolicyViolation,
            None,
            format!("structured extraction validation failed: {}", error),
            0,
        )
    })?;

    let extraction_rows = serde_json::to_value(&extraction.rows).map_err(|error| {
        DocumentError::new(
            "document_extraction_serialization",
            DocumentErrorKind::PolicyViolation,
            None,
            format!("failed serializing extraction rows: {}", error),
            0,
        )
    })?;

    let evidence_packet = json!({
        "schema_version": "1.0.0",
        "produced_by": DOCUMENT_ENGINE_ID,
        "intended_consumers": request.intended_consumers,
        "created_at_ms": request.created_at_ms,
        "trace_id": request.trace_id,
        "query": request.query,
        "retrieved_at_ms": request.now_ms,
        "provider_runs": provider_runs,
        "sources": sources,
        "content_chunks": [],
        "trust_metadata": {
            "document": {
                "schema_id": schema_descriptor.schema_id,
                "doc_norm_version": pdf_text::DOC_NORM_VERSION,
                "rows": extraction_rows,
                "page_count": text_output.page_count,
                "ocr_used": text_output.ocr_used,
                "partial_allowed": schema_descriptor.partial_allowed,
            }
        }
    });

    Ok(DocumentPipelineResult {
        extraction,
        evidence_packet,
    })
}

pub fn parse_document_request(
    tool_request_packet: &Value,
    now_ms: i64,
) -> Result<ParsedDocumentRequest, DocumentError> {
    let object = tool_request_packet.as_object().ok_or_else(|| {
        DocumentError::new(
            "document_parse_request",
            DocumentErrorKind::InvalidInput,
            None,
            "tool request packet must be object",
            0,
        )
    })?;

    let query = object
        .get("query")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| {
            DocumentError::new(
                "document_parse_request",
                DocumentErrorKind::InvalidInput,
                None,
                "tool request query missing",
                0,
            )
        })?
        .to_string();

    let pdf_url = parse_pdf_url(query.as_str()).ok_or_else(|| {
        DocumentError::new(
            "document_parse_request",
            DocumentErrorKind::InsufficientEvidence,
            None,
            "document parser requires explicit pdf URL in query",
            0,
        )
    })?;

    let trace_id = object
        .get("trace_id")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| {
            DocumentError::new(
                "document_parse_request",
                DocumentErrorKind::InvalidInput,
                None,
                "tool request trace_id missing",
                0,
            )
        })?
        .to_string();

    let created_at_ms = object
        .get("created_at_ms")
        .and_then(Value::as_i64)
        .unwrap_or(now_ms);

    let intended_consumers = object
        .get("intended_consumers")
        .and_then(Value::as_array)
        .map(|entries| {
            entries
                .iter()
                .filter_map(Value::as_str)
                .map(|entry| entry.trim().to_string())
                .filter(|entry| !entry.is_empty())
                .collect::<Vec<String>>()
        })
        .filter(|entries| !entries.is_empty())
        .unwrap_or_else(|| {
            vec![
                "PH1.D".to_string(),
                "PH1.WRITE".to_string(),
                "PH1.J".to_string(),
            ]
        });

    let importance_tier = object
        .get("importance_tier")
        .and_then(Value::as_str)
        .unwrap_or("medium")
        .to_string();

    let schema_kind = schema_map::schema_kind_from_tool_request(tool_request_packet);

    Ok(ParsedDocumentRequest {
        trace_id,
        query,
        pdf_url,
        created_at_ms,
        now_ms,
        intended_consumers,
        importance_tier,
        schema_kind,
    })
}

pub fn parse_pdf_url(query: &str) -> Option<String> {
    let trimmed = query.trim();
    if is_pdf_url(trimmed) {
        return Some(trimmed.to_string());
    }

    let marker = "url=";
    let normalized = trimmed.to_ascii_lowercase();
    if let Some(index) = normalized.find(marker) {
        let tail = &trimmed[index + marker.len()..];
        if let Some(candidate) = tail.split_whitespace().next() {
            let candidate = candidate.trim_matches(|ch: char| ch == ';' || ch == ',');
            if is_pdf_url(candidate) {
                return Some(candidate.to_string());
            }
        }
    }

    None
}

fn is_pdf_url(url: &str) -> bool {
    if !(url.starts_with("http://") || url.starts_with("https://")) {
        return false;
    }

    if url.to_ascii_lowercase().ends_with(".pdf") {
        return true;
    }
    url.contains(".pdf?")
}

pub fn ensure_schema_version() -> &'static str {
    STRUCTURED_SCHEMA_VERSION
}

#[cfg(test)]
pub mod document_tests;
