#![forbid(unsafe_code)]

use crate::web_search_plan::structured::types::StructuredRow;
use std::cmp::Ordering;

pub const DEFAULT_WINDOW_MS: i64 = 365_i64 * 24 * 60 * 60 * 1_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AsOfWindow {
    pub from_ms: i64,
    pub to_ms: i64,
}

impl AsOfWindow {
    pub const fn contains(self, timestamp_ms: i64) -> bool {
        timestamp_ms >= self.from_ms && timestamp_ms <= self.to_ms
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AsOfWindowInput {
    pub baseline_from_ms: Option<i64>,
    pub baseline_to_ms: Option<i64>,
    pub compare_from_ms: Option<i64>,
    pub compare_to_ms: Option<i64>,
    pub now_ms: i64,
    pub allow_default_windows: bool,
    pub default_window_ms: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AsOfWindowResolution {
    pub baseline: AsOfWindow,
    pub comparison: AsOfWindow,
    pub used_defaults: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MissingTimestampPolicy {
    Exclude,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WindowFilterResult {
    pub rows: Vec<StructuredRow>,
    pub excluded_missing_timestamp_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AsOfResolutionError {
    pub reason_code: &'static str,
    pub message: String,
}

impl AsOfResolutionError {
    fn new(reason_code: &'static str, message: impl Into<String>) -> Self {
        Self {
            reason_code,
            message: message.into(),
        }
    }
}

pub fn resolve_asof_windows(
    input: &AsOfWindowInput,
) -> Result<AsOfWindowResolution, AsOfResolutionError> {
    let has_all_explicit = input.baseline_from_ms.is_some()
        && input.baseline_to_ms.is_some()
        && input.compare_from_ms.is_some()
        && input.compare_to_ms.is_some();

    if has_all_explicit {
        let baseline = AsOfWindow {
            from_ms: input.baseline_from_ms.unwrap_or_default(),
            to_ms: input.baseline_to_ms.unwrap_or_default(),
        };
        let comparison = AsOfWindow {
            from_ms: input.compare_from_ms.unwrap_or_default(),
            to_ms: input.compare_to_ms.unwrap_or_default(),
        };
        validate_window_pair(baseline, comparison)?;
        return Ok(AsOfWindowResolution {
            baseline,
            comparison,
            used_defaults: false,
        });
    }

    let has_any_explicit = input.baseline_from_ms.is_some()
        || input.baseline_to_ms.is_some()
        || input.compare_from_ms.is_some()
        || input.compare_to_ms.is_some();

    if has_any_explicit {
        return Err(AsOfResolutionError::new(
            "policy_violation",
            "as_of windows must provide baseline and comparison bounds together",
        ));
    }

    if !input.allow_default_windows {
        return Err(AsOfResolutionError::new(
            "insufficient_evidence",
            "explicit as_of windows are required",
        ));
    }

    let window_ms = if input.default_window_ms > 0 {
        input.default_window_ms
    } else {
        DEFAULT_WINDOW_MS
    };
    let comparison = AsOfWindow {
        from_ms: input.now_ms.saturating_sub(window_ms),
        to_ms: input.now_ms,
    };
    let baseline = AsOfWindow {
        from_ms: comparison.from_ms.saturating_sub(window_ms),
        to_ms: comparison.from_ms.saturating_sub(1),
    };
    validate_window_pair(baseline, comparison)?;
    Ok(AsOfWindowResolution {
        baseline,
        comparison,
        used_defaults: true,
    })
}

fn validate_window_pair(
    baseline: AsOfWindow,
    comparison: AsOfWindow,
) -> Result<(), AsOfResolutionError> {
    if baseline.from_ms > baseline.to_ms {
        return Err(AsOfResolutionError::new(
            "policy_violation",
            "baseline from_ms must be <= to_ms",
        ));
    }
    if comparison.from_ms > comparison.to_ms {
        return Err(AsOfResolutionError::new(
            "policy_violation",
            "comparison from_ms must be <= to_ms",
        ));
    }
    if baseline.to_ms >= comparison.from_ms {
        return Err(AsOfResolutionError::new(
            "policy_violation",
            "baseline window must end before comparison window starts",
        ));
    }
    Ok(())
}

pub fn filter_rows_for_window(
    rows: &[StructuredRow],
    window: AsOfWindow,
    missing_timestamp_policy: MissingTimestampPolicy,
) -> WindowFilterResult {
    let mut filtered = Vec::new();
    let mut excluded_missing_timestamp_count = 0usize;

    for row in rows {
        match row.as_of_ms {
            Some(timestamp_ms) if window.contains(timestamp_ms) => filtered.push(row.clone()),
            Some(_) => {}
            None => match missing_timestamp_policy {
                MissingTimestampPolicy::Exclude => {
                    excluded_missing_timestamp_count =
                        excluded_missing_timestamp_count.saturating_add(1);
                }
            },
        }
    }

    filtered.sort_by(|left, right| row_ordering_key(left).cmp(&row_ordering_key(right)));

    WindowFilterResult {
        rows: filtered,
        excluded_missing_timestamp_count,
    }
}

fn row_ordering_key(row: &StructuredRow) -> (String, String, i64, String, String) {
    (
        row.entity.to_ascii_lowercase(),
        row.attribute.to_ascii_lowercase(),
        row.as_of_ms.unwrap_or(i64::MIN),
        row.source_url.to_ascii_lowercase(),
        row.value.ordering_key(),
    )
}

#[allow(dead_code)]
fn cmp_i64(left: i64, right: i64) -> Ordering {
    left.cmp(&right)
}
