#![forbid(unsafe_code)]

use crate::web_search_plan::document::{DocumentError, DocumentErrorKind};
use lopdf::content::Content;
use lopdf::Document;
use lopdf::Object;
use unicode_normalization::UnicodeNormalization;

pub const DOC_NORM_VERSION: &str = "1.0.0";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PdfTextOutput {
    pub extracted_text: String,
    pub page_count: usize,
    pub latency_ms: u64,
    pub doc_norm_version: String,
    pub ocr_used: bool,
}

pub fn extract_text_from_pdf(
    pdf_bytes: &[u8],
    max_extracted_chars: usize,
    _now_ms: i64,
) -> Result<PdfTextOutput, DocumentError> {
    let started = std::time::Instant::now();
    let document = Document::load_mem(pdf_bytes).map_err(|error| {
        DocumentError::new(
            "document_pdf_text",
            DocumentErrorKind::ProviderUpstreamFailed,
            None,
            format!("pdf parse failed: {}", error),
            0,
        )
    })?;

    let pages = document.get_pages();
    let page_count = pages.len();
    if page_count == 0 {
        return Err(DocumentError::new(
            "document_pdf_text",
            DocumentErrorKind::ProviderUpstreamFailed,
            None,
            "pdf has zero pages",
            0,
        ));
    }

    let mut raw_text = String::new();
    for page_id in pages.values() {
        let content_data = document.get_page_content(*page_id).map_err(|error| {
            DocumentError::new(
                "document_pdf_text",
                DocumentErrorKind::ProviderUpstreamFailed,
                None,
                format!("pdf page content read failed: {}", error),
                0,
            )
        })?;
        let content = Content::decode(&content_data).map_err(|error| {
            DocumentError::new(
                "document_pdf_text",
                DocumentErrorKind::ProviderUpstreamFailed,
                None,
                format!("pdf content decode failed: {}", error),
                0,
            )
        })?;

        for operation in content.operations {
            match operation.operator.as_str() {
                "Tj" => {
                    if let Some(text) = operation.operands.first().and_then(object_text) {
                        raw_text.push_str(text.as_str());
                    }
                }
                "TJ" => {
                    if let Some(Object::Array(parts)) = operation.operands.first() {
                        for part in parts {
                            if let Some(text) = object_text(part) {
                                raw_text.push_str(text.as_str());
                            }
                        }
                    }
                }
                "'" => {
                    raw_text.push('\n');
                    if let Some(text) = operation.operands.first().and_then(object_text) {
                        raw_text.push_str(text.as_str());
                    }
                }
                "\"" => {
                    raw_text.push('\n');
                    if let Some(text) = operation.operands.get(2).and_then(object_text) {
                        raw_text.push_str(text.as_str());
                    }
                }
                "Td" | "TD" | "T*" => {
                    raw_text.push('\n');
                }
                _ => {}
            }
        }
        raw_text.push('\n');
    }

    if raw_text.trim().is_empty() {
        let page_numbers = pages.keys().copied().collect::<Vec<u32>>();
        raw_text = document.extract_text(&page_numbers).map_err(|error| {
            DocumentError::new(
                "document_pdf_text",
                DocumentErrorKind::ProviderUpstreamFailed,
                None,
                format!("pdf text extraction failed: {}", error),
                0,
            )
        })?;
    }

    let normalized = normalize_document_text(raw_text.as_str());
    let bounded = normalized
        .chars()
        .take(max_extracted_chars)
        .collect::<String>();

    Ok(PdfTextOutput {
        extracted_text: bounded,
        page_count,
        latency_ms: started.elapsed().as_millis() as u64,
        doc_norm_version: DOC_NORM_VERSION.to_string(),
        ocr_used: false,
    })
}

fn object_text(object: &Object) -> Option<String> {
    match object {
        Object::String(bytes, _) => Some(String::from_utf8_lossy(bytes).trim().to_string()),
        Object::Name(name) => Some(String::from_utf8_lossy(name).trim().to_string()),
        _ => None,
    }
}

pub fn normalize_document_text(input: &str) -> String {
    let mut normalized_lines = Vec::new();
    let normalized_unicode = input
        .replace("\r\n", "\n")
        .replace('\r', "\n")
        .replace('\u{00A0}', " ")
        .nfc()
        .collect::<String>();

    for line in normalized_unicode.lines() {
        let collapsed = line
            .split_whitespace()
            .collect::<Vec<&str>>()
            .join(" ")
            .trim()
            .to_string();
        if !collapsed.is_empty() {
            normalized_lines.push(collapsed);
        }
    }

    normalized_lines.join("\n")
}
