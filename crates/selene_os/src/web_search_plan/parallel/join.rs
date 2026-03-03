#![forbid(unsafe_code)]

use std::collections::BTreeMap;

pub fn join_in_planned_order<T: Clone>(
    planned_task_ids: &[String],
    completed: &BTreeMap<String, T>,
) -> Vec<T> {
    planned_task_ids
        .iter()
        .filter_map(|task_id| completed.get(task_id).cloned())
        .collect()
}
