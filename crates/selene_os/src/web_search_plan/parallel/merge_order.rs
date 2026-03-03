#![forbid(unsafe_code)]

use crate::web_search_plan::parallel::join::join_in_planned_order;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MergeItem<T> {
    pub canonical_url: String,
    pub payload: T,
}

pub fn merge_lead_then_fallback<T: Clone>(
    lead: &[MergeItem<T>],
    fallback: &[MergeItem<T>],
) -> Vec<MergeItem<T>> {
    let mut merged = Vec::new();
    let mut seen = BTreeSet::new();

    for item in lead {
        if seen.insert(item.canonical_url.clone()) {
            merged.push(item.clone());
        }
    }

    for item in fallback {
        if seen.insert(item.canonical_url.clone()) {
            merged.push(item.clone());
        }
    }

    merged
}

pub fn merge_completed_by_plan<T: Clone>(
    planned_task_ids: &[String],
    completed: &BTreeMap<String, MergeItem<T>>,
) -> Vec<MergeItem<T>> {
    join_in_planned_order(planned_task_ids, completed)
}
