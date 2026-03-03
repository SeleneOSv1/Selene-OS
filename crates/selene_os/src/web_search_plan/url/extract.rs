#![forbid(unsafe_code)]

use crate::web_search_plan::url::charset::normalize_text_v1;
use crate::web_search_plan::url::mime::AllowedMime;
use crate::web_search_plan::url::UrlFetchErrorKind;

pub const EXTRACTION_VERSION: &str = "1.0.0";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtractedDocument {
    pub title: String,
    pub body_text: String,
    pub extraction_chars: usize,
}

pub fn extract_document(
    mime: AllowedMime,
    decoded_text: &str,
    max_chars: usize,
) -> Result<ExtractedDocument, UrlFetchErrorKind> {
    let (title_raw, body_raw) = match mime {
        AllowedMime::Html | AllowedMime::Xhtml => extract_html(decoded_text),
        AllowedMime::Plain | AllowedMime::Pdf => (String::new(), decoded_text.to_string()),
    };

    let title = normalize_text_v1(&decode_basic_html_entities(&title_raw));
    let body_text = normalize_text_v1(&decode_basic_html_entities(&body_raw));
    let body_chars = body_text.chars().count();

    if body_chars > max_chars {
        return Err(UrlFetchErrorKind::ExtractionTooLarge);
    }

    Ok(ExtractedDocument {
        title,
        body_text,
        extraction_chars: body_chars,
    })
}

fn extract_html(input: &str) -> (String, String) {
    let without_script = strip_tag_block_case_insensitive(input, "script");
    let sanitized = strip_tag_block_case_insensitive(&without_script, "style");
    let title = capture_title_case_insensitive(&sanitized).unwrap_or_default();
    let body = strip_html_tags(&sanitized);
    (title, body)
}

fn strip_tag_block_case_insensitive(input: &str, tag: &str) -> String {
    let lower = input.to_ascii_lowercase();
    let open = format!("<{}", tag);
    let close = format!("</{}>", tag);

    let mut out = String::with_capacity(input.len());
    let mut idx = 0usize;
    while let Some(open_rel) = lower[idx..].find(&open) {
        let open_idx = idx + open_rel;
        out.push_str(&input[idx..open_idx]);
        if let Some(close_rel) = lower[open_idx..].find(&close) {
            let close_idx = open_idx + close_rel + close.len();
            idx = close_idx;
        } else {
            idx = input.len();
            break;
        }
    }
    if idx < input.len() {
        out.push_str(&input[idx..]);
    }
    out
}

fn capture_title_case_insensitive(input: &str) -> Option<String> {
    let lower = input.to_ascii_lowercase();
    let start_tag = "<title";
    let start = lower.find(start_tag)?;
    let open_end_rel = lower[start..].find('>')?;
    let content_start = start + open_end_rel + 1;
    let end_rel = lower[content_start..].find("</title>")?;
    let content_end = content_start + end_rel;
    Some(input[content_start..content_end].to_string())
}

fn strip_html_tags(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut in_tag = false;
    for ch in input.chars() {
        match ch {
            '<' => {
                in_tag = true;
            }
            '>' => {
                in_tag = false;
                out.push(' ');
            }
            _ if !in_tag => out.push(ch),
            _ => {}
        }
    }
    out
}

fn decode_basic_html_entities(input: &str) -> String {
    input
        .replace("&nbsp;", " ")
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
}
