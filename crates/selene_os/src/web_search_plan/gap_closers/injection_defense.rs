#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

pub const INJECTION_DEFENSE_VERSION: &str = "1.0.0";
pub const MATERIAL_REDUCTION_THRESHOLD_BP: u32 = 2_000;

const DENYLIST_PATTERNS: [&str; 8] = [
    "ignore previous instructions",
    "system prompt",
    "developer message",
    "exfiltrate",
    "api key",
    "password",
    "override policy",
    "tool call",
];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FlaggedSegment {
    pub line_index: usize,
    pub matched_pattern: String,
    pub text_excerpt: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InjectionDefenseOutcome {
    pub defense_version: String,
    pub sanitized_text: String,
    pub flagged_segments: Vec<FlaggedSegment>,
    pub removed_char_count: usize,
    pub materially_reduced: bool,
    pub reason_code: Option<String>,
}

pub fn sanitize_fetched_content(raw_text: &str) -> InjectionDefenseOutcome {
    let mut kept_lines = Vec::new();
    let mut flagged_segments = Vec::new();
    let mut removed_char_count = 0usize;

    for (line_index, line) in raw_text.lines().enumerate() {
        let lower = line.to_ascii_lowercase();
        let matched_pattern = DENYLIST_PATTERNS
            .iter()
            .find(|pattern| lower.contains(**pattern))
            .copied();

        if let Some(pattern) = matched_pattern {
            removed_char_count = removed_char_count.saturating_add(line.chars().count());
            flagged_segments.push(FlaggedSegment {
                line_index,
                matched_pattern: pattern.to_string(),
                text_excerpt: bounded_excerpt(line, 160),
            });
            continue;
        }
        kept_lines.push(line.to_string());
    }

    let sanitized_text = kept_lines.join("\n");
    let total_chars = raw_text.chars().count();
    let materially_reduced = if total_chars == 0 {
        false
    } else {
        let removed_bp = ((removed_char_count as u128) * 10_000u128) / (total_chars as u128);
        removed_bp >= MATERIAL_REDUCTION_THRESHOLD_BP as u128
    };

    let reason_code = if flagged_segments.is_empty() {
        None
    } else if sanitized_text.trim().is_empty() {
        Some("insufficient_evidence".to_string())
    } else if materially_reduced {
        Some("policy_violation".to_string())
    } else {
        None
    };

    InjectionDefenseOutcome {
        defense_version: INJECTION_DEFENSE_VERSION.to_string(),
        sanitized_text,
        flagged_segments,
        removed_char_count,
        materially_reduced,
        reason_code,
    }
}

fn bounded_excerpt(input: &str, max_chars: usize) -> String {
    input.chars().take(max_chars).collect::<String>()
}
