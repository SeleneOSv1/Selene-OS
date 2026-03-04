#![forbid(unsafe_code)]

use crate::web_search_plan::merge::delta::{ChangeType, DeltaChange, ExternalFinding};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub const EXTERNAL_EVIDENCE_PREVAILS: &str = "external_evidence_prevails";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConflictItem {
    pub topic_key: String,
    pub internal_claim: String,
    pub evidence_claim: String,
    pub evidence_citations: Vec<String>,
    pub resolution_rule: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConflictReport {
    pub conflicts: Vec<ConflictItem>,
}

pub fn build_conflict_report(
    delta_changes: &[DeltaChange],
    external_findings: &[ExternalFinding],
) -> Option<ConflictReport> {
    let mut conflicts = Vec::new();

    for change in delta_changes {
        if change.change_type != ChangeType::Contradicted {
            continue;
        }
        conflicts.push(ConflictItem {
            topic_key: change.topic_key.clone(),
            internal_claim: change.prior_statement.clone(),
            evidence_claim: change.new_statement.clone(),
            evidence_citations: change.citations.clone(),
            resolution_rule: EXTERNAL_EVIDENCE_PREVAILS.to_string(),
        });
    }

    let grouped = group_contradictions(external_findings);
    for (group_id, statements) in grouped {
        if statements.len() < 2 {
            continue;
        }
        for statement in statements {
            conflicts.push(ConflictItem {
                topic_key: group_id.clone(),
                internal_claim: String::new(),
                evidence_claim: statement.statement,
                evidence_citations: statement.citations,
                resolution_rule: EXTERNAL_EVIDENCE_PREVAILS.to_string(),
            });
        }
    }

    if conflicts.is_empty() {
        return None;
    }

    conflicts.sort_by(|left, right| {
        (
            left.topic_key.as_str(),
            left.evidence_claim.as_str(),
            left.evidence_citations.first().map(String::as_str).unwrap_or(""),
        )
            .cmp(&(
                right.topic_key.as_str(),
                right.evidence_claim.as_str(),
                right.evidence_citations.first().map(String::as_str).unwrap_or(""),
            ))
    });
    conflicts.dedup_by(|left, right| {
        left.topic_key == right.topic_key
            && left.internal_claim == right.internal_claim
            && left.evidence_claim == right.evidence_claim
            && left.evidence_citations == right.evidence_citations
    });

    Some(ConflictReport { conflicts })
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct GroupedConflictStatement {
    statement: String,
    citations: Vec<String>,
}

fn group_contradictions(
    external_findings: &[ExternalFinding],
) -> BTreeMap<String, Vec<GroupedConflictStatement>> {
    let mut grouped = BTreeMap::<String, Vec<GroupedConflictStatement>>::new();
    for finding in external_findings {
        let Some(group_id) = finding.contradiction_group_id.as_ref() else {
            continue;
        };
        grouped
            .entry(group_id.clone())
            .or_default()
            .push(GroupedConflictStatement {
                statement: finding.statement.clone(),
                citations: finding.citations.clone(),
            });
    }

    for entries in grouped.values_mut() {
        let mut dedup_key_set = BTreeSet::new();
        entries.retain(|entry| {
            let key = (
                entry.statement.as_str(),
                entry.citations.first().map(String::as_str).unwrap_or(""),
            );
            dedup_key_set.insert((key.0.to_string(), key.1.to_string()))
        });
        entries.sort_by(|left, right| {
            (
                left.statement.as_str(),
                left.citations.first().map(String::as_str).unwrap_or(""),
            )
                .cmp(&(
                    right.statement.as_str(),
                    right.citations.first().map(String::as_str).unwrap_or(""),
                ))
        });
    }

    grouped
}
