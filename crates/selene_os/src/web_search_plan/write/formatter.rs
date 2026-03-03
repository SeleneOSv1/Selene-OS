#![forbid(unsafe_code)]

use crate::web_search_plan::write::citation_renderer::{
    build_citation_map, citation_keys_for_bullet, parse_evidence_lines, strip_marker_tokens,
};
use crate::web_search_plan::write::style_guard::{
    normalize_bullet_text, normalize_claim_text, normalize_direct_answer, validate_style_guard,
    StyleGuardConfig,
};
use crate::web_search_plan::write::{WriteError, WriteFormatMode};
use serde_json::{Map, Value};

#[derive(Debug, Clone)]
pub struct FormattedWrite {
    pub formatted_text: String,
    pub citation_map: Map<String, Value>,
    pub citation_count: usize,
    pub bullet_count: usize,
    pub style_guard_passed: bool,
}

pub fn format_synthesis_packet(
    synthesis_packet: &Value,
    format_mode: WriteFormatMode,
) -> Result<FormattedWrite, WriteError> {
    let synthesis_object = synthesis_packet.as_object().ok_or_else(|| {
        WriteError::InvalidSynthesis("synthesis packet must be object".to_string())
    })?;

    let answer_text = synthesis_object
        .get("answer_text")
        .and_then(Value::as_str)
        .ok_or_else(|| {
            WriteError::InvalidSynthesis("synthesis packet missing answer_text".to_string())
        })?;

    let synthesis_citations = synthesis_object
        .get("citations")
        .and_then(Value::as_array)
        .ok_or_else(|| {
            WriteError::InvalidSynthesis("synthesis packet missing citations".to_string())
        })?;

    let bullet_evidence: Vec<String> = synthesis_object
        .get("bullet_evidence")
        .and_then(Value::as_array)
        .ok_or_else(|| {
            WriteError::InvalidSynthesis("synthesis packet missing bullet_evidence".to_string())
        })?
        .iter()
        .map(|value| {
            value
                .as_str()
                .ok_or_else(|| {
                    WriteError::InvalidSynthesis(
                        "synthesis bullet_evidence entries must be strings".to_string(),
                    )
                })
                .map(ToString::to_string)
        })
        .collect::<Result<Vec<String>, WriteError>>()?;

    let reason_codes: Vec<String> = synthesis_object
        .get("reason_codes")
        .and_then(Value::as_array)
        .ok_or_else(|| {
            WriteError::InvalidSynthesis("synthesis packet missing reason_codes".to_string())
        })?
        .iter()
        .map(|value| {
            value
                .as_str()
                .ok_or_else(|| {
                    WriteError::InvalidSynthesis(
                        "synthesis reason_codes entries must be strings".to_string(),
                    )
                })
                .map(ToString::to_string)
        })
        .collect::<Result<Vec<String>, WriteError>>()?;

    if reason_codes.iter().any(|code| code == "unsupported_claim") {
        return Err(WriteError::UnsupportedClaim(
            "synthesis packet includes unsupported_claim reason code".to_string(),
        ));
    }

    let uncertainty_flags: Vec<String> = synthesis_object
        .get("uncertainty_flags")
        .and_then(Value::as_array)
        .ok_or_else(|| {
            WriteError::InvalidSynthesis("synthesis packet missing uncertainty_flags".to_string())
        })?
        .iter()
        .map(|value| {
            value
                .as_str()
                .ok_or_else(|| {
                    WriteError::InvalidSynthesis(
                        "synthesis uncertainty_flags entries must be strings".to_string(),
                    )
                })
                .map(ToString::to_string)
        })
        .collect::<Result<Vec<String>, WriteError>>()?;

    let parsed_evidence_lines = parse_evidence_lines(answer_text);
    if parsed_evidence_lines.is_empty() {
        return Err(WriteError::CitationMismatch(
            "synthesis answer_text evidence section has no bulleted lines".to_string(),
        ));
    }

    if parsed_evidence_lines.len() != bullet_evidence.len() {
        return Err(WriteError::StyleGuardViolation(
            "evidence bullet count mismatch between answer_text and bullet_evidence".to_string(),
        ));
    }

    let direct_answer_raw = extract_direct_answer(answer_text).ok_or_else(|| {
        WriteError::InvalidSynthesis("unable to extract direct answer section".to_string())
    })?;

    let direct_answer = normalize_direct_answer(
        &direct_answer_raw,
        format_mode.max_direct_answer_sentences(),
    );
    if direct_answer.is_empty() {
        return Err(WriteError::StyleGuardViolation(
            "direct answer became empty after normalization".to_string(),
        ));
    }

    let citation_build = build_citation_map(answer_text, synthesis_citations)
        .map_err(WriteError::CitationMismatch)?;

    let mut rendered_bullets = Vec::new();
    for (index, parsed_line) in parsed_evidence_lines.iter().enumerate() {
        let source_bullet_text = normalize_claim_text(&bullet_evidence[index]);
        let parsed_bullet_text = normalize_claim_text(&strip_marker_tokens(&parsed_line.text));
        if source_bullet_text != parsed_bullet_text {
            return Err(WriteError::StyleGuardViolation(
                "semantic mutation detected between synthesis bullet_evidence and evidence lines"
                    .to_string(),
            ));
        }

        let citation_keys =
            citation_keys_for_bullet(&parsed_line.citations, &citation_build.key_by_ref);
        if citation_keys.is_empty() {
            return Err(WriteError::CitationMismatch(format!(
                "evidence bullet {} has no source_url citation",
                index + 1
            )));
        }

        let normalized_bullet = normalize_bullet_text(&source_bullet_text);
        rendered_bullets.push(format!(
            "- {} [{}]",
            normalized_bullet,
            citation_keys.join(",")
        ));
    }

    if citation_build.ordered_url_keys.is_empty() {
        return Err(WriteError::CitationMismatch(
            "no source_url citations available for citation list".to_string(),
        ));
    }

    let mut lines = Vec::new();
    lines.push("Direct Answer:".to_string());
    lines.push(direct_answer.clone());
    lines.push(String::new());
    lines.push("Evidence:".to_string());
    lines.extend(rendered_bullets.iter().cloned());
    lines.push(String::new());
    lines.push("Citations:".to_string());

    for key in &citation_build.ordered_url_keys {
        let Some(entry) = citation_build.citation_map.get(key) else {
            return Err(WriteError::CitationMismatch(format!(
                "citation key {} missing from citation_map",
                key
            )));
        };

        let Some(url) = entry.get("value").and_then(Value::as_str) else {
            return Err(WriteError::CitationMismatch(format!(
                "citation key {} missing string value",
                key
            )));
        };

        lines.push(format!("- [{}] {}", key, url));
    }

    let formatted_text = lines.join("\n");

    validate_style_guard(
        &direct_answer,
        &bullet_evidence,
        &bullet_evidence,
        &uncertainty_flags,
        &formatted_text,
        StyleGuardConfig {
            max_direct_answer_sentences: format_mode.max_direct_answer_sentences(),
            ..StyleGuardConfig::default()
        },
    )
    .map_err(WriteError::StyleGuardViolation)?;

    Ok(FormattedWrite {
        formatted_text,
        citation_count: citation_build.citation_map.len(),
        bullet_count: rendered_bullets.len(),
        citation_map: citation_build.citation_map,
        style_guard_passed: true,
    })
}

fn extract_direct_answer(answer_text: &str) -> Option<String> {
    let mut in_direct_answer = false;
    let mut lines = Vec::new();

    for raw_line in answer_text.lines() {
        let line = raw_line.trim();

        if line.eq_ignore_ascii_case("Direct Answer") || line.eq_ignore_ascii_case("Direct Answer:")
        {
            in_direct_answer = true;
            continue;
        }

        if !in_direct_answer {
            continue;
        }

        if line.eq_ignore_ascii_case("Evidence") || line.eq_ignore_ascii_case("Evidence:") {
            break;
        }

        if line.is_empty() {
            if !lines.is_empty() {
                break;
            }
            continue;
        }

        lines.push(line.to_string());
    }

    if lines.is_empty() {
        return None;
    }

    Some(lines.join(" "))
}
