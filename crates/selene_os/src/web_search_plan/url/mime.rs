#![forbid(unsafe_code)]

use crate::web_search_plan::url::UrlFetchErrorKind;

pub const MIME_SNIFF_PREFIX_BYTES: usize = 512;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AllowedMime {
    Html,
    Plain,
    Xhtml,
    Pdf,
}

impl AllowedMime {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Html => "text/html",
            Self::Plain => "text/plain",
            Self::Xhtml => "application/xhtml+xml",
            Self::Pdf => "application/pdf",
        }
    }
}

pub fn detect_allowed_mime(
    content_type_header: Option<&str>,
    sniff_bytes: &[u8],
) -> Result<AllowedMime, UrlFetchErrorKind> {
    if let Some(header) = content_type_header {
        let main = normalize_content_type(header);
        if let Some(allowed) = allowed_for_type(main.as_str()) {
            return Ok(allowed);
        }
        if is_disallowed_binary(main.as_str()) {
            return Err(UrlFetchErrorKind::MimeNotAllowed);
        }
        return Err(UrlFetchErrorKind::MimeNotAllowed);
    }

    sniff_mime(sniff_bytes)
}

fn normalize_content_type(raw: &str) -> String {
    raw.split(';')
        .next()
        .unwrap_or("")
        .trim()
        .to_ascii_lowercase()
}

fn allowed_for_type(main: &str) -> Option<AllowedMime> {
    match main {
        "text/html" => Some(AllowedMime::Html),
        "text/plain" => Some(AllowedMime::Plain),
        "application/xhtml+xml" => Some(AllowedMime::Xhtml),
        "application/pdf" => Some(AllowedMime::Pdf),
        _ => None,
    }
}

fn is_disallowed_binary(main: &str) -> bool {
    matches!(
        main,
        "application/octet-stream"
            | "application/zip"
            | "application/x-zip-compressed"
            | "application/x-rar-compressed"
            | "application/x-7z-compressed"
            | "application/x-tar"
            | "application/gzip"
            | "application/x-gzip"
            | "application/x-msdownload"
            | "application/x-dosexec"
    )
}

fn sniff_mime(sniff_bytes: &[u8]) -> Result<AllowedMime, UrlFetchErrorKind> {
    let trimmed = trim_ascii_prefix(sniff_bytes);
    if trimmed.starts_with(b"%PDF-") {
        return Ok(AllowedMime::Pdf);
    }

    let lower = String::from_utf8_lossy(trimmed).to_ascii_lowercase();
    if lower.starts_with("<!doctype html") || lower.starts_with("<html") {
        return Ok(AllowedMime::Html);
    }
    if lower.starts_with("<?xml") && lower.contains("<html") {
        return Ok(AllowedMime::Xhtml);
    }

    if looks_like_text(trimmed) {
        return Ok(AllowedMime::Plain);
    }

    Err(UrlFetchErrorKind::MimeAmbiguous)
}

fn trim_ascii_prefix(bytes: &[u8]) -> &[u8] {
    let mut idx = 0usize;
    while idx < bytes.len() && bytes[idx].is_ascii_whitespace() {
        idx = idx.saturating_add(1);
    }
    &bytes[idx..]
}

fn looks_like_text(bytes: &[u8]) -> bool {
    if bytes.is_empty() {
        return false;
    }
    let mut printable = 0usize;
    let mut total = 0usize;
    for byte in bytes {
        total = total.saturating_add(1);
        if *byte == b'\n' || *byte == b'\r' || *byte == b'\t' || (32..=126).contains(byte) {
            printable = printable.saturating_add(1);
        }
    }
    printable.saturating_mul(100) / total >= 90
}
