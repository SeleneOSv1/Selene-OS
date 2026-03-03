#![forbid(unsafe_code)]

const FORBIDDEN_FILLER: &[&str] = &[
    "as an ai",
    "i think",
    "i believe",
    "it is worth noting",
    "clearly",
    "obviously",
    "definitely",
];

const FORBIDDEN_SPECULATIVE: &[&str] = &[
    "maybe", "perhaps", "possibly", "might", "could be", "probably", "likely",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StyleGuardConfig {
    pub max_direct_answer_sentences: usize,
    pub max_sentence_words: usize,
    pub min_bullet_chars: usize,
}

impl Default for StyleGuardConfig {
    fn default() -> Self {
        Self {
            max_direct_answer_sentences: 4,
            max_sentence_words: 36,
            min_bullet_chars: 12,
        }
    }
}

pub fn normalize_claim_text(input: &str) -> String {
    normalize_whitespace_and_punctuation(input)
}

pub fn normalize_direct_answer(input: &str, max_sentences: usize) -> String {
    let capped = if max_sentences == 0 { 1 } else { max_sentences };
    let mut out = Vec::new();
    for sentence in split_sentences(input).into_iter().take(capped) {
        let mut normalized = normalize_whitespace_and_punctuation(&sentence);
        if normalized.is_empty() {
            continue;
        }
        normalized = uppercase_first_alpha(&normalized);
        if !normalized.ends_with('.') && !normalized.ends_with('!') && !normalized.ends_with('?') {
            normalized.push('.');
        }
        out.push(normalized);
    }

    out.join(" ")
}

pub fn normalize_bullet_text(input: &str) -> String {
    let mut normalized = normalize_whitespace_and_punctuation(input);
    if normalized.is_empty() {
        return normalized;
    }
    normalized = uppercase_first_alpha(&normalized);
    if !normalized.ends_with('.') && !normalized.ends_with('!') && !normalized.ends_with('?') {
        normalized.push('.');
    }
    normalized
}

pub fn validate_style_guard(
    direct_answer: &str,
    evidence_bullets: &[String],
    source_bullet_evidence: &[String],
    uncertainty_flags: &[String],
    formatted_text: &str,
    config: StyleGuardConfig,
) -> Result<(), String> {
    ensure_required_headings(formatted_text)?;
    ensure_no_markdown_artifacts(formatted_text)?;
    ensure_deterministic_whitespace(formatted_text)?;

    let direct_sentences = split_sentences(direct_answer);
    if direct_sentences.is_empty() {
        return Err("direct answer must contain at least one sentence".to_string());
    }
    if direct_sentences.len() > config.max_direct_answer_sentences {
        return Err(format!(
            "direct answer exceeds max sentence count {}",
            config.max_direct_answer_sentences
        ));
    }

    for sentence in &direct_sentences {
        let words = sentence
            .split_whitespace()
            .filter(|token| !token.trim().is_empty())
            .count();
        if words > config.max_sentence_words {
            return Err(format!(
                "direct answer sentence exceeds max words {}",
                config.max_sentence_words
            ));
        }
    }

    let normalized_full = formatted_text.to_ascii_lowercase();
    for phrase in FORBIDDEN_FILLER {
        if normalized_full.contains(phrase) {
            return Err(format!("forbidden filler phrase detected: {}", phrase));
        }
    }

    if uncertainty_flags.is_empty() {
        let normalized_direct = direct_answer.to_ascii_lowercase();
        for phrase in FORBIDDEN_SPECULATIVE {
            if normalized_direct.contains(phrase) {
                return Err(format!(
                    "speculative language not allowed without uncertainty flags: {}",
                    phrase
                ));
            }
        }
    }

    if evidence_bullets.len() != source_bullet_evidence.len() {
        return Err("evidence bullet count differs from synthesis bullet evidence".to_string());
    }

    for (rendered, source) in evidence_bullets.iter().zip(source_bullet_evidence.iter()) {
        if rendered.chars().count() < config.min_bullet_chars {
            return Err("evidence bullet too short for clarity threshold".to_string());
        }

        let rendered_norm = normalize_claim_text(rendered);
        let source_norm = normalize_claim_text(source);
        if rendered_norm != source_norm {
            return Err("semantic mutation detected in evidence bullet".to_string());
        }
    }

    if formatted_text.contains("??")
        || formatted_text.contains("!!")
        || formatted_text.contains("...")
    {
        return Err("unsupported punctuation pattern detected".to_string());
    }

    Ok(())
}

fn ensure_required_headings(formatted_text: &str) -> Result<(), String> {
    for heading in ["Direct Answer:", "Evidence:", "Citations:"] {
        if !formatted_text.contains(heading) {
            return Err(format!("missing required heading {}", heading));
        }
    }
    Ok(())
}

fn ensure_no_markdown_artifacts(formatted_text: &str) -> Result<(), String> {
    for marker in ["**", "__", "```", "`", "# ", "> "] {
        if formatted_text.contains(marker) {
            return Err("markdown artifact detected in formatted output".to_string());
        }
    }
    Ok(())
}

fn ensure_deterministic_whitespace(formatted_text: &str) -> Result<(), String> {
    if formatted_text.contains('\t') {
        return Err("tabs are not allowed in deterministic formatted text".to_string());
    }
    if formatted_text.contains("  ") {
        return Err("double spaces are not allowed in deterministic formatted text".to_string());
    }
    Ok(())
}

fn normalize_whitespace_and_punctuation(input: &str) -> String {
    input
        .replace('“', "\"")
        .replace('”', "\"")
        .replace('’', "'")
        .split_whitespace()
        .collect::<Vec<&str>>()
        .join(" ")
        .trim()
        .to_string()
}

fn uppercase_first_alpha(input: &str) -> String {
    let mut chars = input.chars();
    let mut out = String::with_capacity(input.len());

    while let Some(ch) = chars.next() {
        if ch.is_ascii_alphabetic() {
            out.push(ch.to_ascii_uppercase());
            break;
        }
        out.push(ch);
    }

    out.push_str(chars.as_str());
    out
}

fn split_sentences(input: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut current = String::new();

    for ch in input.chars() {
        current.push(ch);
        if matches!(ch, '.' | '!' | '?') {
            let normalized = normalize_whitespace_and_punctuation(&current);
            if !normalized.is_empty() {
                out.push(normalized);
            }
            current.clear();
        }
    }

    let trailing = normalize_whitespace_and_punctuation(&current);
    if !trailing.is_empty() {
        out.push(trailing);
    }

    out
}
