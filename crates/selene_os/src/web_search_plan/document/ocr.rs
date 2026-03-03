#![forbid(unsafe_code)]

use crate::web_search_plan::document::pdf_text::normalize_document_text;
use crate::web_search_plan::document::{DocumentError, DocumentErrorKind};
use std::fs;
use std::io::Write;
use std::process::Command;
use std::time::Instant;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OcrOptions {
    pub backend: Option<String>,
    pub language: String,
    pub max_pages: usize,
}

impl OcrOptions {
    pub fn from_env() -> Self {
        Self {
            backend: std::env::var("SELENE_DOCUMENT_OCR_BACKEND")
                .ok()
                .map(|value| value.trim().to_ascii_lowercase())
                .filter(|value| !value.is_empty()),
            language: std::env::var("SELENE_DOCUMENT_OCR_LANGUAGE")
                .ok()
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty())
                .unwrap_or_else(|| "eng".to_string()),
            max_pages: std::env::var("SELENE_DOCUMENT_OCR_MAX_PAGES")
                .ok()
                .and_then(|value| value.trim().parse::<usize>().ok())
                .filter(|value| *value > 0)
                .unwrap_or(1),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OcrOutput {
    pub extracted_text: String,
    pub pages_processed: usize,
    pub latency_ms: u64,
}

pub fn extract_text_from_pdf_with_ocr(
    pdf_bytes: &[u8],
    options: &OcrOptions,
    max_extracted_chars: usize,
) -> Result<OcrOutput, DocumentError> {
    let backend = options.backend.as_deref().ok_or_else(|| {
        DocumentError::new(
            "document_ocr",
            DocumentErrorKind::ProviderUnconfigured,
            None,
            "OCR backend not configured",
            0,
        )
    })?;

    if backend != "tesseract" {
        return Err(DocumentError::new(
            "document_ocr",
            DocumentErrorKind::ProviderUnconfigured,
            None,
            format!("OCR backend {} is not supported", backend),
            0,
        ));
    }

    let started = Instant::now();
    let timestamp_ns = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or(0);
    let temp_root = std::env::temp_dir().join(format!(
        "selene_document_ocr_{}_{}",
        std::process::id(),
        timestamp_ns
    ));
    fs::create_dir_all(&temp_root).map_err(|error| {
        DocumentError::new(
            "document_ocr",
            DocumentErrorKind::ProviderUpstreamFailed,
            None,
            format!("failed creating OCR temp directory: {}", error),
            0,
        )
    })?;

    let input_pdf = temp_root.join("input.pdf");
    let mut input_file = fs::File::create(&input_pdf).map_err(|error| {
        DocumentError::new(
            "document_ocr",
            DocumentErrorKind::ProviderUpstreamFailed,
            None,
            format!("failed creating OCR input file: {}", error),
            0,
        )
    })?;
    input_file.write_all(pdf_bytes).map_err(|error| {
        DocumentError::new(
            "document_ocr",
            DocumentErrorKind::ProviderUpstreamFailed,
            None,
            format!("failed writing OCR input file: {}", error),
            0,
        )
    })?;

    let image_prefix = temp_root.join("page");
    let pdftoppm_status = Command::new("pdftoppm")
        .arg("-f")
        .arg("1")
        .arg("-l")
        .arg(options.max_pages.to_string())
        .arg("-singlefile")
        .arg("-png")
        .arg(&input_pdf)
        .arg(&image_prefix)
        .output();

    let pdftoppm_output = pdftoppm_status.map_err(|error| {
        DocumentError::new(
            "document_ocr",
            DocumentErrorKind::ProviderUpstreamFailed,
            None,
            format!("pdftoppm invocation failed: {}", error),
            0,
        )
    })?;
    if !pdftoppm_output.status.success() {
        return Err(DocumentError::new(
            "document_ocr",
            DocumentErrorKind::ProviderUpstreamFailed,
            None,
            format!(
                "pdftoppm failed: {}",
                String::from_utf8_lossy(&pdftoppm_output.stderr)
            ),
            started.elapsed().as_millis() as u64,
        ));
    }

    let image_path = temp_root.join("page.png");
    let tesseract_output = Command::new("tesseract")
        .arg(&image_path)
        .arg("stdout")
        .arg("-l")
        .arg(options.language.as_str())
        .arg("--psm")
        .arg("6")
        .output()
        .map_err(|error| {
            DocumentError::new(
                "document_ocr",
                DocumentErrorKind::ProviderUpstreamFailed,
                None,
                format!("tesseract invocation failed: {}", error),
                started.elapsed().as_millis() as u64,
            )
        })?;

    if !tesseract_output.status.success() {
        return Err(DocumentError::new(
            "document_ocr",
            DocumentErrorKind::ProviderUpstreamFailed,
            None,
            format!(
                "tesseract failed: {}",
                String::from_utf8_lossy(&tesseract_output.stderr)
            ),
            started.elapsed().as_millis() as u64,
        ));
    }

    let normalized = normalize_document_text(&String::from_utf8_lossy(&tesseract_output.stdout));
    let bounded = normalized
        .chars()
        .take(max_extracted_chars)
        .collect::<String>();

    let _ = fs::remove_file(temp_root.join("page.png"));
    let _ = fs::remove_file(&input_pdf);
    let _ = fs::remove_dir_all(&temp_root);

    Ok(OcrOutput {
        extracted_text: bounded,
        pages_processed: options.max_pages,
        latency_ms: started.elapsed().as_millis() as u64,
    })
}
