#![forbid(unsafe_code)]

use crate::web_search_plan::write::style_guard::normalize_claim_text;

const DOMINANCE_THRESHOLD: f64 = 0.85;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ScriptLanguage {
    En,
    Zh,
}

impl ScriptLanguage {
    const fn as_tag(self) -> &'static str {
        match self {
            Self::En => "en",
            Self::Zh => "zh-CN",
        }
    }
}

pub fn derive_language_tag(direct_answer: &str, evidence_bullets: &[String]) -> String {
    let mut aggregate = String::new();
    aggregate.push_str(direct_answer);
    aggregate.push('\n');
    for bullet in evidence_bullets {
        aggregate.push_str(bullet);
        aggregate.push('\n');
    }

    match classify_segment_language(&aggregate) {
        Some(language) => language.as_tag().to_string(),
        None => "en".to_string(),
    }
}

pub fn enforce_language_contract(
    language_tag: &str,
    direct_answer: &str,
    evidence_bullets: &[String],
) -> Result<(), String> {
    let expected = normalize_language_tag(language_tag)?;

    for sentence in split_segments(direct_answer) {
        ensure_expected_language(expected, &sentence)?;
    }

    for bullet in evidence_bullets {
        ensure_expected_language(expected, bullet)?;
    }

    Ok(())
}

pub fn normalize_language_tag(language_tag: &str) -> Result<&'static str, String> {
    let normalized = language_tag.trim().to_ascii_lowercase();
    if normalized.starts_with("en") {
        return Ok("en");
    }
    if normalized.starts_with("zh") {
        return Ok("zh-CN");
    }
    Err(format!("unsupported language tag {}", language_tag))
}

fn ensure_expected_language(expected_tag: &str, segment: &str) -> Result<(), String> {
    let normalized = normalize_claim_text(segment);
    if normalized.is_empty() {
        return Ok(());
    }

    let Some(detected) = classify_segment_language(&normalized) else {
        return Ok(());
    };

    if detected.as_tag() != expected_tag {
        return Err(format!(
            "language contract violation: expected {} but detected {} in segment '{}'",
            expected_tag,
            detected.as_tag(),
            normalized
        ));
    }

    Ok(())
}

fn classify_segment_language(segment: &str) -> Option<ScriptLanguage> {
    let mut latin_count = 0usize;
    let mut cjk_count = 0usize;

    for ch in segment.chars() {
        if ch.is_ascii_alphabetic() {
            latin_count += 1;
            continue;
        }
        if is_cjk(ch) {
            cjk_count += 1;
        }
    }

    if latin_count == 0 && cjk_count == 0 {
        return None;
    }
    if latin_count == 0 {
        return Some(ScriptLanguage::Zh);
    }
    if cjk_count == 0 {
        return Some(ScriptLanguage::En);
    }

    let total = (latin_count + cjk_count) as f64;
    let latin_ratio = latin_count as f64 / total;
    let cjk_ratio = cjk_count as f64 / total;
    if latin_ratio >= DOMINANCE_THRESHOLD {
        return Some(ScriptLanguage::En);
    }
    if cjk_ratio >= DOMINANCE_THRESHOLD {
        return Some(ScriptLanguage::Zh);
    }

    None
}

fn split_segments(input: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut current = String::new();

    for ch in input.chars() {
        current.push(ch);
        if matches!(ch, '.' | '!' | '?' | '。' | '！' | '？') {
            out.push(current.trim().to_string());
            current.clear();
        }
    }

    let tail = current.trim();
    if !tail.is_empty() {
        out.push(tail.to_string());
    }

    out
}

fn is_cjk(ch: char) -> bool {
    matches!(
        ch as u32,
        0x3400..=0x4DBF // CJK Unified Ideographs Extension A
            | 0x4E00..=0x9FFF // CJK Unified Ideographs
            | 0xF900..=0xFAFF // CJK Compatibility Ideographs
            | 0x20000..=0x2A6DF // CJK Unified Ideographs Extension B
            | 0x2A700..=0x2B73F // Extension C
            | 0x2B740..=0x2B81F // Extension D
            | 0x2B820..=0x2CEAF // Extension E/F
    )
}
