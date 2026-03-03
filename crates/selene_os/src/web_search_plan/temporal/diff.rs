#![forbid(unsafe_code)]

use crate::web_search_plan::temporal::change_classify::{classify_change, ChangeType};
use crate::web_search_plan::temporal::timeline::{structured_to_temporal_value, TemporalValue};
use crate::web_search_plan::structured::types::{StructuredRow, StructuredValue};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChangeItem {
    pub key: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prior_value: Option<TemporalValue>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub new_value: Option<TemporalValue>,
    pub change_type: ChangeType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delta_value: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_seen_ms: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latest_seen_ms: Option<i64>,
    pub citations_prior: Vec<String>,
    pub citations_new: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiffBuildResult {
    pub changes: Vec<ChangeItem>,
    pub reason_codes: Vec<String>,
    pub unit_mismatch_count: usize,
    pub modified_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SnapshotRow {
    key: String,
    entity: String,
    attribute: String,
    unit: Option<String>,
    currency: Option<String>,
    value: TemporalValue,
    as_of_ms: i64,
    source_url: String,
    source_refs: Vec<String>,
}

pub fn build_changes(
    baseline_rows: &[StructuredRow],
    comparison_rows: &[StructuredRow],
) -> DiffBuildResult {
    let baseline = build_window_snapshot(baseline_rows);
    let comparison = build_window_snapshot(comparison_rows);

    let mut reason_codes = Vec::new();
    let mut changes = Vec::new();
    let mut unit_mismatch_count = 0usize;
    let mut modified_count = 0usize;

    let mut keys = baseline
        .keys()
        .chain(comparison.keys())
        .cloned()
        .collect::<Vec<String>>();
    keys.sort();
    keys.dedup();

    for key in keys {
        let prior = baseline.get(key.as_str());
        let next = comparison.get(key.as_str());

        let unit_matches = prior
            .and_then(|row| row.unit.as_ref())
            .eq(&next.and_then(|row| row.unit.as_ref()));
        let currency_matches = prior
            .and_then(|row| row.currency.as_ref())
            .eq(&next.and_then(|row| row.currency.as_ref()));

        let classification = classify_change(
            prior.map(|row| &row.value),
            next.map(|row| &row.value),
            unit_matches,
            currency_matches,
        );

        if classification.change_type == ChangeType::Modified {
            modified_count = modified_count.saturating_add(1);
        }
        if classification.unit_mismatch {
            unit_mismatch_count = unit_mismatch_count.saturating_add(1);
            push_reason_code(&mut reason_codes, "policy_violation");
        }

        let first_seen_ms = min_option_i64(prior.map(|row| row.as_of_ms), next.map(|row| row.as_of_ms));
        let latest_seen_ms = max_option_i64(prior.map(|row| row.as_of_ms), next.map(|row| row.as_of_ms));

        let citations_prior = prior
            .map(|row| row.source_refs.clone())
            .unwrap_or_default();
        let citations_new = next
            .map(|row| row.source_refs.clone())
            .unwrap_or_default();

        let item = ChangeItem {
            key: key.clone(),
            prior_value: prior.map(|row| row.value.clone()),
            new_value: next.map(|row| row.value.clone()),
            change_type: classification.change_type,
            delta_value: classification.delta_value,
            first_seen_ms,
            latest_seen_ms,
            citations_prior,
            citations_new,
        };
        changes.push(item);
    }

    changes.sort_by(|left, right| {
        (
            left.key.clone(),
            left.change_type,
            left.latest_seen_ms.unwrap_or(i64::MIN),
        )
            .cmp(&(
                right.key.clone(),
                right.change_type,
                right.latest_seen_ms.unwrap_or(i64::MIN),
            ))
    });

    DiffBuildResult {
        changes,
        reason_codes,
        unit_mismatch_count,
        modified_count,
    }
}

fn build_window_snapshot(rows: &[StructuredRow]) -> BTreeMap<String, SnapshotRow> {
    let mut out = BTreeMap::new();

    for row in rows {
        let Some(as_of_ms) = row.as_of_ms else {
            continue;
        };

        let (unit, currency) = unit_currency_from_row(row);
        let key = snapshot_key(row.entity.as_str(), row.attribute.as_str(), unit.as_deref());

        let mut source_refs = vec![row.source_ref.clone(), row.source_url.clone()];
        source_refs.sort();
        source_refs.dedup();
        let candidate = SnapshotRow {
            key: key.clone(),
            entity: row.entity.clone(),
            attribute: row.attribute.clone(),
            unit,
            currency,
            value: structured_to_temporal_value(&row.value),
            as_of_ms,
            source_url: row.source_url.clone(),
            source_refs,
        };

        match out.get_mut(key.as_str()) {
            Some(existing) => {
                if should_replace(existing, &candidate) {
                    *existing = candidate;
                }
            }
            None => {
                out.insert(key, candidate);
            }
        }
    }

    out
}

fn should_replace(existing: &SnapshotRow, candidate: &SnapshotRow) -> bool {
    if candidate.as_of_ms > existing.as_of_ms {
        return true;
    }
    if candidate.as_of_ms < existing.as_of_ms {
        return false;
    }

    let candidate_source = candidate.source_url.to_ascii_lowercase();
    let existing_source = existing.source_url.to_ascii_lowercase();
    if candidate_source < existing_source {
        return true;
    }
    if candidate_source > existing_source {
        return false;
    }

    candidate.value.ordering_key() < existing.value.ordering_key()
}

fn snapshot_key(entity: &str, attribute: &str, unit: Option<&str>) -> String {
    format!(
        "{}|{}|{}",
        entity.to_ascii_lowercase(),
        attribute.to_ascii_lowercase(),
        unit.unwrap_or("")
    )
}

fn unit_currency_from_row(row: &StructuredRow) -> (Option<String>, Option<String>) {
    match &row.value {
        StructuredValue::Currency { currency_code, .. } => {
            (row.unit.clone(), Some(currency_code.to_ascii_uppercase()))
        }
        _ => (row.unit.clone(), None),
    }
}

fn push_reason_code(reason_codes: &mut Vec<String>, reason_code: &str) {
    if !reason_codes.iter().any(|entry| entry == reason_code) {
        reason_codes.push(reason_code.to_string());
    }
}

fn min_option_i64(left: Option<i64>, right: Option<i64>) -> Option<i64> {
    match (left, right) {
        (Some(a), Some(b)) => Some(a.min(b)),
        (Some(a), None) => Some(a),
        (None, Some(b)) => Some(b),
        (None, None) => None,
    }
}

fn max_option_i64(left: Option<i64>, right: Option<i64>) -> Option<i64> {
    match (left, right) {
        (Some(a), Some(b)) => Some(a.max(b)),
        (Some(a), None) => Some(a),
        (None, Some(b)) => Some(b),
        (None, None) => None,
    }
}
