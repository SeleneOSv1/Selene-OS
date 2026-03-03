#![forbid(unsafe_code)]

use crate::web_search_plan::url::UrlFetchErrorKind;

pub const QUALITY_GATE_VERSION: &str = "1.0.0";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct QualityMetrics {
    pub text_len: usize,
    pub noise_ratio_bp: u32,
}

pub fn evaluate_text_quality(
    text: &str,
    min_text_length: usize,
    max_noise_ratio_bp: u32,
) -> Result<QualityMetrics, UrlFetchErrorKind> {
    let text_len = text.chars().count();
    if text_len < min_text_length {
        return Err(UrlFetchErrorKind::ExtractionQualityLow);
    }

    let mut noise = 0usize;
    let mut measured = 0usize;
    for ch in text.chars() {
        if ch.is_whitespace() {
            continue;
        }
        measured = measured.saturating_add(1);
        if is_noise_char(ch) {
            noise = noise.saturating_add(1);
        }
    }
    if measured == 0 {
        return Err(UrlFetchErrorKind::EmptyExtraction);
    }

    let noise_ratio_bp = ((noise.saturating_mul(10_000)) / measured) as u32;
    if noise_ratio_bp > max_noise_ratio_bp {
        return Err(UrlFetchErrorKind::ExtractionQualityLow);
    }

    Ok(QualityMetrics {
        text_len,
        noise_ratio_bp,
    })
}

fn is_noise_char(ch: char) -> bool {
    if ch.is_ascii_alphanumeric() {
        return false;
    }
    matches!(
        ch,
        '~' | '^' | '`' | '|' | '\\' | '{' | '}' | '<' | '>' | '@' | '#' | '$' | '%'
    )
}
