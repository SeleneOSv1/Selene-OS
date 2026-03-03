#![forbid(unsafe_code)]

use crate::web_search_plan::document::{
    DocumentError, DocumentErrorKind, DocumentRuntimeConfig, ParsedDocumentRequest,
};
use crate::web_search_plan::perf_cost::tiers::ImportanceTier;
use crate::web_search_plan::perf_cost::timeouts::clamp_provider_timeout;
use crate::web_search_plan::proxy::ProxyMode;
use std::io::Read;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PdfFetchOutput {
    pub bytes: Vec<u8>,
    pub final_url: String,
    pub mime_type: String,
    pub latency_ms: u64,
}

pub fn fetch_pdf(
    request: &ParsedDocumentRequest,
    config: &DocumentRuntimeConfig,
) -> Result<PdfFetchOutput, DocumentError> {
    let tier = ImportanceTier::parse_or_default(request.importance_tier.as_str());
    let timeout_ms = clamp_provider_timeout(config.pdf_fetch_timeout_ms, tier);

    let mut builder = ureq::AgentBuilder::new()
        .timeout_connect(Duration::from_millis(timeout_ms))
        .timeout_read(Duration::from_millis(timeout_ms))
        .timeout_write(Duration::from_millis(timeout_ms))
        .user_agent("selene-document-parser/1.0")
        .try_proxy_from_env(false)
        .redirects(3);

    if let Some(proxy_url) = proxy_for_url(request.pdf_url.as_str(), config) {
        let proxy = ureq::Proxy::new(proxy_url).map_err(|_| {
            DocumentError::new(
                "document_pdf_fetch",
                DocumentErrorKind::PolicyViolation,
                None,
                "invalid proxy configuration for document fetch",
                0,
            )
        })?;
        builder = builder.proxy(proxy);
    }

    let started = Instant::now();
    let response = match builder
        .build()
        .get(request.pdf_url.as_str())
        .set("Accept", "application/pdf")
        .timeout(Duration::from_millis(timeout_ms))
        .call()
    {
        Ok(response) => response,
        Err(ureq::Error::Status(code, _)) => {
            return Err(DocumentError::new(
                "document_pdf_fetch",
                DocumentErrorKind::ProviderUpstreamFailed,
                Some(code),
                format!("document fetch returned http status {}", code),
                started.elapsed().as_millis() as u64,
            ));
        }
        Err(ureq::Error::Transport(error)) => {
            let message = format!("{:?} {}", error.kind(), error).to_ascii_lowercase();
            let kind = if message.contains("timeout") {
                DocumentErrorKind::TimeoutExceeded
            } else {
                DocumentErrorKind::ProviderUpstreamFailed
            };
            return Err(DocumentError::new(
                "document_pdf_fetch",
                kind,
                None,
                "document fetch transport failure",
                started.elapsed().as_millis() as u64,
            ));
        }
    };

    let mime_type = response
        .header("content-type")
        .unwrap_or_default()
        .to_ascii_lowercase();
    if !mime_type.starts_with("application/pdf") {
        return Err(DocumentError::new(
            "document_pdf_fetch",
            DocumentErrorKind::PolicyViolation,
            None,
            format!("document fetch rejected MIME {}", mime_type),
            started.elapsed().as_millis() as u64,
        ));
    }

    let final_url = response.get_url().trim().to_string();
    let mut body = Vec::new();
    let mut reader = response.into_reader().take(config.max_pdf_bytes as u64 + 1);
    reader.read_to_end(&mut body).map_err(|error| {
        DocumentError::new(
            "document_pdf_fetch",
            DocumentErrorKind::ProviderUpstreamFailed,
            None,
            format!("failed reading PDF body: {}", error),
            started.elapsed().as_millis() as u64,
        )
    })?;

    if body.len() > config.max_pdf_bytes {
        return Err(DocumentError::new(
            "document_pdf_fetch",
            DocumentErrorKind::PolicyViolation,
            None,
            format!("pdf exceeded max_pdf_bytes {}", config.max_pdf_bytes),
            started.elapsed().as_millis() as u64,
        ));
    }

    Ok(PdfFetchOutput {
        bytes: body,
        final_url,
        mime_type,
        latency_ms: started.elapsed().as_millis() as u64,
    })
}

fn proxy_for_url<'a>(url: &str, config: &'a DocumentRuntimeConfig) -> Option<&'a str> {
    let is_https = url.to_ascii_lowercase().starts_with("https://");
    match config.proxy_config.mode {
        ProxyMode::Off => None,
        ProxyMode::Env | ProxyMode::Explicit => {
            if is_https {
                config.proxy_config.https_proxy_url.as_deref()
            } else {
                config.proxy_config.http_proxy_url.as_deref()
            }
        }
    }
}
