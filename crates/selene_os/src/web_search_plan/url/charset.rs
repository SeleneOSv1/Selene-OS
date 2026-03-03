#![forbid(unsafe_code)]

use encoding_rs::{Encoding, UTF_8};
use unicode_normalization::UnicodeNormalization;

pub const CHARSET_VERSION: &str = "1.0.0";
pub const NORMALIZATION_VERSION: &str = "1.0.0";
pub const CHARSET_SNIFF_LIMIT_BYTES: usize = 16 * 1024;

pub fn select_charset(content_type_header: Option<&str>, sniff_bytes: &[u8]) -> &'static Encoding {
    if let Some(enc) = charset_from_content_type(content_type_header) {
        return enc;
    }
    if let Some(enc) = charset_from_html_meta(sniff_bytes) {
        return enc;
    }
    UTF_8
}

pub fn charset_from_content_type(content_type_header: Option<&str>) -> Option<&'static Encoding> {
    let header = content_type_header?;
    for part in header.split(';').skip(1) {
        let trimmed = part.trim();
        let Some((left, right)) = trimmed.split_once('=') else {
            continue;
        };
        if left.trim().eq_ignore_ascii_case("charset") {
            let label = right.trim().trim_matches('"').to_ascii_lowercase();
            if let Some(enc) = Encoding::for_label(label.as_bytes()) {
                return Some(enc);
            }
        }
    }
    None
}

pub fn charset_from_html_meta(sniff_bytes: &[u8]) -> Option<&'static Encoding> {
    let lower = String::from_utf8_lossy(sniff_bytes).to_ascii_lowercase();
    let marker = "charset=";
    let idx = lower.find(marker)?;
    let rest = lower[idx + marker.len()..]
        .trim_start_matches(|ch: char| ch == '"' || ch == '\'' || ch.is_ascii_whitespace());
    let mut end = 0usize;
    for ch in rest.chars() {
        if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' || ch == '.' {
            end = end.saturating_add(ch.len_utf8());
        } else {
            break;
        }
    }
    if end == 0 {
        return None;
    }
    Encoding::for_label(rest[..end].as_bytes())
}

pub fn normalize_line_endings_v1(input: &str) -> String {
    input.replace("\r\n", "\n").replace('\r', "\n")
}

pub fn collapse_whitespace_v1(input: &str) -> String {
    input.split_whitespace().collect::<Vec<&str>>().join(" ")
}

pub fn normalize_text_v1(input: &str) -> String {
    let normalized_lines = normalize_line_endings_v1(input);
    let unicode_normalized: String = normalized_lines.nfc().collect();
    collapse_whitespace_v1(&unicode_normalized)
}
