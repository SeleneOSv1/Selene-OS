#![forbid(unsafe_code)]

use crate::web_search_plan::temporal::timeline::TemporalValue;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChangeType {
    Added,
    Removed,
    Modified,
    Unchanged,
}

impl ChangeType {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Added => "added",
            Self::Removed => "removed",
            Self::Modified => "modified",
            Self::Unchanged => "unchanged",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChangeClassification {
    pub change_type: ChangeType,
    pub delta_value: Option<String>,
    pub comparable: bool,
    pub unit_mismatch: bool,
}

pub fn classify_change(
    prior: Option<&TemporalValue>,
    next: Option<&TemporalValue>,
    unit_matches: bool,
    currency_matches: bool,
) -> ChangeClassification {
    match (prior, next) {
        (None, Some(_)) => ChangeClassification {
            change_type: ChangeType::Added,
            delta_value: None,
            comparable: false,
            unit_mismatch: false,
        },
        (Some(_), None) => ChangeClassification {
            change_type: ChangeType::Removed,
            delta_value: None,
            comparable: false,
            unit_mismatch: false,
        },
        (None, None) => ChangeClassification {
            change_type: ChangeType::Unchanged,
            delta_value: None,
            comparable: false,
            unit_mismatch: false,
        },
        (Some(prior_value), Some(next_value)) => {
            if !unit_matches || !currency_matches {
                return ChangeClassification {
                    change_type: ChangeType::Modified,
                    delta_value: None,
                    comparable: false,
                    unit_mismatch: true,
                };
            }

            if prior_value == next_value {
                return ChangeClassification {
                    change_type: ChangeType::Unchanged,
                    delta_value: Some("0".to_string()),
                    comparable: true,
                    unit_mismatch: false,
                };
            }

            let delta = numeric_delta(prior_value, next_value);
            ChangeClassification {
                change_type: ChangeType::Modified,
                delta_value: delta,
                comparable: true,
                unit_mismatch: false,
            }
        }
    }
}

fn numeric_delta(prior: &TemporalValue, next: &TemporalValue) -> Option<String> {
    let prior_value = temporal_numeric(prior)?;
    let next_value = temporal_numeric(next)?;
    Some((next_value - prior_value).normalize().to_string())
}

fn temporal_numeric(value: &TemporalValue) -> Option<Decimal> {
    match value {
        TemporalValue::Int { value } => Some(Decimal::from(*value)),
        TemporalValue::Decimal { value } => value.parse::<Decimal>().ok(),
        TemporalValue::Currency { amount, .. } => amount.parse::<Decimal>().ok(),
        TemporalValue::Percent { value } => value.parse::<Decimal>().ok(),
        TemporalValue::String { .. } | TemporalValue::Bool { .. } | TemporalValue::Date { .. } => {
            None
        }
    }
}
