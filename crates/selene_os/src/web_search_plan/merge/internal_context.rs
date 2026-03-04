#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

pub const MAX_PRIOR_SUMMARY_CHARS: usize = 1024;
pub const MAX_PRIOR_KEY_POINTS: usize = 16;
pub const MAX_PRIOR_KEY_POINT_CHARS: usize = 512;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InternalSourceType {
    Memory,
    PriorReport,
    PriorSession,
}

impl InternalSourceType {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Memory => "memory",
            Self::PriorReport => "prior_report",
            Self::PriorSession => "prior_session",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InternalContext {
    pub prior_summary: String,
    pub prior_key_points: Vec<String>,
    pub prior_timestamp_ms: i64,
    pub internal_source_type: InternalSourceType,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InternalView {
    pub prior_summary: String,
    pub prior_key_points: Vec<String>,
    pub prior_timestamp_ms: i64,
    pub internal_source_type: InternalSourceType,
}

pub fn build_internal_view(internal_context: Option<&InternalContext>) -> InternalView {
    let Some(context) = internal_context else {
        return InternalView {
            prior_summary: String::new(),
            prior_key_points: Vec::new(),
            prior_timestamp_ms: 0,
            internal_source_type: InternalSourceType::PriorSession,
        };
    };

    let prior_summary = normalize_text(context.prior_summary.as_str(), MAX_PRIOR_SUMMARY_CHARS);
    let mut prior_key_points = context
        .prior_key_points
        .iter()
        .map(|entry| normalize_text(entry.as_str(), MAX_PRIOR_KEY_POINT_CHARS))
        .filter(|entry| !entry.is_empty())
        .collect::<Vec<String>>();
    prior_key_points.sort();
    prior_key_points.dedup();
    if prior_key_points.len() > MAX_PRIOR_KEY_POINTS {
        prior_key_points.truncate(MAX_PRIOR_KEY_POINTS);
    }

    InternalView {
        prior_summary,
        prior_key_points,
        prior_timestamp_ms: context.prior_timestamp_ms,
        internal_source_type: context.internal_source_type,
    }
}

pub fn internal_context_used(view: &InternalView) -> bool {
    !view.prior_summary.is_empty() || !view.prior_key_points.is_empty()
}

fn normalize_text(raw: &str, max_chars: usize) -> String {
    let normalized = raw
        .split_whitespace()
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .collect::<Vec<&str>>()
        .join(" ");

    if normalized.chars().count() <= max_chars {
        normalized
    } else {
        normalized
            .chars()
            .take(max_chars)
            .collect::<String>()
            .trim()
            .to_string()
    }
}
