#![forbid(unsafe_code)]

use serde_json::{json, Value};
use std::collections::BTreeSet;

pub const STITCHING_POLICY_VERSION: &str = "run33-stitching-v1";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContradictionSummary {
    pub agrees: Vec<String>,
    pub conflicts: Vec<String>,
    pub unknowns: Vec<String>,
}

pub fn stitch_sources(primary: &[Value], fallback: &[Value]) -> Vec<Value> {
    let mut stitched = Vec::new();
    let mut seen_keys = BTreeSet::new();

    for source in primary.iter().chain(fallback.iter()) {
        let key = source_key(source);
        if key.is_empty() || !seen_keys.insert(key) {
            continue;
        }
        stitched.push(source.clone());
    }

    stitched
}

pub fn build_stitching_summary(
    source_titles: &[String],
    open_failure_urls: &[String],
    reason_codes: &[String],
) -> ContradictionSummary {
    let agrees = source_titles
        .iter()
        .map(|title| normalize_line(title))
        .filter(|title| !title.is_empty())
        .take(2)
        .collect::<Vec<String>>();

    let has_conflict = reason_codes
        .iter()
        .any(|code| code == "conflicting_evidence_detected");
    let conflicts = if has_conflict {
        source_titles
            .iter()
            .map(|title| normalize_line(title))
            .filter(|title| !title.is_empty())
            .take(2)
            .collect::<Vec<String>>()
    } else {
        Vec::new()
    };

    let unknowns = open_failure_urls
        .iter()
        .map(|url| normalize_line(url))
        .filter(|url| !url.is_empty())
        .collect::<Vec<String>>();

    ContradictionSummary {
        agrees,
        conflicts,
        unknowns,
    }
}

pub fn deep_mode_contradiction_lines(
    bullet_evidence: &[String],
    uncertainty_flags: &[String],
    reason_codes: &[String],
) -> Vec<String> {
    let has_conflict_flag = uncertainty_flags
        .iter()
        .any(|flag| flag == "conflict_detected" || flag == "conflict_present");
    let has_conflict_reason = reason_codes
        .iter()
        .any(|code| code == "conflicting_evidence_detected");

    if !has_conflict_flag && !has_conflict_reason {
        return Vec::new();
    }

    let normalized_bullets = bullet_evidence
        .iter()
        .map(|entry| normalize_line(entry))
        .filter(|entry| !entry.is_empty())
        .collect::<Vec<String>>();

    let agrees = normalized_bullets
        .first()
        .cloned()
        .unwrap_or_else(|| "No consistent agreement could be established.".to_string());
    let conflicts = normalized_bullets
        .iter()
        .take(2)
        .cloned()
        .collect::<Vec<String>>()
        .join(" | ");
    let conflicts = if conflicts.is_empty() {
        "Conflicting claims were detected.".to_string()
    } else {
        conflicts
    };

    vec![
        format!("- Agrees: {}", agrees),
        format!("- Conflicts: {}", conflicts),
        "- Unknown: Additional corroborated evidence is required.".to_string(),
    ]
}

pub fn contradiction_summary_json(summary: &ContradictionSummary) -> Value {
    json!({
        "agrees": summary.agrees,
        "conflicts": summary.conflicts,
        "unknowns": summary.unknowns,
    })
}

fn source_key(source: &Value) -> String {
    source
        .get("canonical_url")
        .and_then(Value::as_str)
        .or_else(|| source.get("url").and_then(Value::as_str))
        .unwrap_or_default()
        .to_ascii_lowercase()
}

fn normalize_line(input: &str) -> String {
    input
        .split_whitespace()
        .collect::<Vec<&str>>()
        .join(" ")
        .trim()
        .to_string()
}
