#![forbid(unsafe_code)]

#[derive(Clone, Copy, PartialEq, Eq)]
enum VoiceSection {
    DirectAnswer,
    Evidence,
    Contradictions,
    OptionalUncertainty,
    Unknown,
}

pub fn render_voice_output(formatted_text: &str) -> String {
    let mut lines = Vec::new();
    let mut previous_was_pause = false;
    let mut current_section = VoiceSection::Unknown;

    for raw in formatted_text.lines() {
        let clean = strip_markdown_artifacts(raw).trim().to_string();
        if clean.is_empty() {
            if !previous_was_pause && !lines.is_empty() {
                lines.push("[PAUSE_LONG]".to_string());
                previous_was_pause = true;
            }
            continue;
        }

        if let Some(section) = heading_section(&clean) {
            if clean == "Citations:" {
                break;
            }
            current_section = section;
            lines.push(clean);
            lines.push("[PAUSE_SHORT]".to_string());
            previous_was_pause = true;
            continue;
        }

        if clean.ends_with(':') {
            current_section = VoiceSection::Unknown;
            continue;
        }

        if current_section == VoiceSection::Unknown {
            continue;
        }

        let is_bullet = clean.starts_with("- ");
        let body = if is_bullet {
            clean.trim_start_matches("- ").trim()
        } else {
            clean.as_str()
        };

        let Some(safe_body) = sanitize_voice_body(body) else {
            continue;
        };

        previous_was_pause = false;
        if is_bullet {
            lines.push(format!("• {}", safe_body));
        } else {
            lines.push(safe_body);
        }
    }

    lines.join("\n")
}

fn heading_section(line: &str) -> Option<VoiceSection> {
    match line {
        "Direct Answer:" => Some(VoiceSection::DirectAnswer),
        "Evidence:" => Some(VoiceSection::Evidence),
        "Contradictions:" => Some(VoiceSection::Contradictions),
        "Optional Uncertainty:" => Some(VoiceSection::OptionalUncertainty),
        "Citations:" => Some(VoiceSection::Unknown),
        _ => None,
    }
}

fn strip_markdown_artifacts(input: &str) -> String {
    input
        .replace("**", "")
        .replace("__", "")
        .replace('`', "")
        .replace('#', "")
        .replace('>', "")
}

fn sanitize_voice_body(input: &str) -> Option<String> {
    let normalized = input
        .split_whitespace()
        .filter(|token| !is_display_only_token(token))
        .collect::<Vec<&str>>()
        .join(" ");
    let normalized = normalized.trim();

    if normalized.is_empty() || is_display_only_metadata_line(normalized) {
        return None;
    }

    Some(normalized.to_string())
}

fn is_display_only_token(token: &str) -> bool {
    let trimmed = token.trim_matches(|ch: char| matches!(ch, ',' | ';'));
    let lower = trimmed.to_ascii_lowercase();

    if lower.starts_with("http://") || lower.starts_with("https://") {
        return true;
    }

    if lower.starts_with("[url:") || lower.starts_with("[chunk:") {
        return true;
    }

    is_citation_key_token(trimmed)
}

fn is_citation_key_token(token: &str) -> bool {
    let Some(inner) = token.strip_prefix('[').and_then(|value| value.strip_suffix(']')) else {
        return false;
    };
    if inner.is_empty() {
        return false;
    }

    inner
        .split(',')
        .all(|item| item.trim().starts_with('C') && item.trim()[1..].chars().all(|ch| ch.is_ascii_digit()))
}

fn is_display_only_metadata_line(line: &str) -> bool {
    let lower = line.to_ascii_lowercase();
    [
        "trace_id:",
        "debug:",
        "debug packet:",
        "provider_json:",
        "provider payload:",
        "packet_hash:",
        "packet_hashes:",
        "response_hash:",
        "evidence_hash:",
        "reason_codes:",
        "policy_snapshot_id:",
        "created_at_ms:",
        "schema_version:",
        "produced_by:",
        "intended_consumers:",
    ]
    .iter()
    .any(|prefix| lower.starts_with(prefix))
}
