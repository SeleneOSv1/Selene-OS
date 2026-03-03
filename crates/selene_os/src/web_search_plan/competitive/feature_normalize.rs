#![forbid(unsafe_code)]

use crate::web_search_plan::competitive::entity_normalize::EntityIndex;
use crate::web_search_plan::competitive::schema::{
    CompetitiveError, CompetitiveErrorKind, CompetitiveFeature, TriState,
};
use crate::web_search_plan::structured::types::{StructuredRow, StructuredValue};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug)]
struct FeatureAccumulator {
    feature_label: String,
    presence: TriState,
    source_refs: BTreeSet<String>,
}

impl Default for FeatureAccumulator {
    fn default() -> Self {
        Self {
            feature_label: String::new(),
            presence: TriState::Unknown,
            source_refs: BTreeSet::new(),
        }
    }
}

pub fn build_feature_matrix(
    rows: &[StructuredRow],
    entities: &EntityIndex,
    allowed_source_refs: &BTreeSet<String>,
) -> Result<Vec<CompetitiveFeature>, CompetitiveError> {
    let mut accumulators: BTreeMap<(String, String), FeatureAccumulator> = BTreeMap::new();

    for row in rows {
        if !is_feature_row(row) {
            continue;
        }
        let Some(entity_id) = entities.entity_id_for_name(row.entity.as_str()) else {
            continue;
        };
        let feature_key = normalize_feature_key(row.attribute.as_str());
        if feature_key.is_empty() {
            continue;
        }
        let feature_label = normalize_feature_label(row.attribute.as_str(), feature_key.as_str());
        let presence = parse_presence(&row.value);
        let source_ref = row.source_ref.trim().to_string();
        if source_ref.is_empty() && presence != TriState::Unknown {
            return Err(CompetitiveError::new(
                CompetitiveErrorKind::PolicyViolation,
                format!(
                    "feature row {}:{} has concrete presence without source_ref",
                    row.entity, row.attribute
                ),
            ));
        }
        if !source_ref.is_empty() && !allowed_source_refs.contains(&source_ref) {
            return Err(CompetitiveError::new(
                CompetitiveErrorKind::CitationMismatch,
                format!(
                    "feature source_ref {} is not present in evidence packet",
                    source_ref
                ),
            ));
        }

        let key = (entity_id.to_string(), feature_key.clone());
        let entry = accumulators.entry(key).or_default();
        if entry.feature_label.is_empty() {
            entry.feature_label = feature_label;
        }
        entry.presence = merge_presence(entry.presence, presence);
        if !source_ref.is_empty() {
            entry.source_refs.insert(source_ref);
        }
    }

    let mut features = Vec::with_capacity(accumulators.len());
    for ((entity_id, feature_key), accumulator) in accumulators {
        if accumulator.presence != TriState::Unknown && accumulator.source_refs.is_empty() {
            return Err(CompetitiveError::new(
                CompetitiveErrorKind::PolicyViolation,
                format!(
                    "feature {} for {} lacks source_refs for concrete presence",
                    feature_key, entity_id
                ),
            ));
        }

        features.push(CompetitiveFeature {
            entity_id,
            feature_key,
            feature_label: accumulator.feature_label,
            presence: accumulator.presence,
            source_refs: accumulator.source_refs.into_iter().collect(),
        });
    }

    features.sort_by(|left, right| {
        left.feature_key
            .cmp(&right.feature_key)
            .then_with(|| left.entity_id.cmp(&right.entity_id))
            .then_with(|| left.feature_label.cmp(&right.feature_label))
    });
    Ok(features)
}

pub fn normalize_feature_key(raw: &str) -> String {
    let lower = raw.trim().to_lowercase();
    let trimmed = lower.strip_prefix("feature:").unwrap_or(lower.as_str());
    let mut key = String::with_capacity(trimmed.len());
    let mut last_was_underscore = false;
    for ch in trimmed.chars() {
        if ch.is_ascii_alphanumeric() {
            key.push(ch);
            last_was_underscore = false;
        } else if !last_was_underscore {
            key.push('_');
            last_was_underscore = true;
        }
    }
    key.trim_matches('_').to_string()
}

fn normalize_feature_label(raw: &str, fallback_key: &str) -> String {
    let lower = raw.trim().to_lowercase();
    let trimmed = lower.strip_prefix("feature:").unwrap_or(lower.as_str()).trim();
    if trimmed.is_empty() {
        fallback_key.to_string()
    } else {
        trimmed.to_string()
    }
}

fn is_feature_row(row: &StructuredRow) -> bool {
    let attribute = row.attribute.to_lowercase();
    attribute.starts_with("feature:")
        || attribute.contains("feature")
        || matches!(row.value, StructuredValue::Bool { .. })
}

fn parse_presence(value: &StructuredValue) -> TriState {
    match value {
        StructuredValue::Bool { value } => {
            if *value {
                TriState::True
            } else {
                TriState::False
            }
        }
        StructuredValue::Int { value } => {
            if *value > 0 {
                TriState::True
            } else {
                TriState::False
            }
        }
        StructuredValue::Float { value } => {
            if *value > 0.0 {
                TriState::True
            } else {
                TriState::False
            }
        }
        StructuredValue::String { value } => {
            let normalized = value.trim().to_lowercase();
            match normalized.as_str() {
                "yes" | "true" | "supported" | "available" | "included" => TriState::True,
                "no" | "false" | "unsupported" | "not supported" | "excluded" => TriState::False,
                _ => TriState::Unknown,
            }
        }
        _ => TriState::Unknown,
    }
}

fn merge_presence(current: TriState, incoming: TriState) -> TriState {
    match (current, incoming) {
        (TriState::Unknown, other) => other,
        (existing, TriState::Unknown) => existing,
        (TriState::True, TriState::True) => TriState::True,
        (TriState::False, TriState::False) => TriState::False,
        _ => TriState::Unknown,
    }
}
