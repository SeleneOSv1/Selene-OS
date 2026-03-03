#![forbid(unsafe_code)]

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RetrievalTask {
    pub task_id: String,
    pub priority: u32,
    pub canonical_url: String,
    pub provider_id: String,
    pub task_type: String,
}

impl RetrievalTask {
    pub fn stable_key(&self) -> String {
        build_stable_key(
            self.canonical_url.as_str(),
            self.provider_id.as_str(),
            self.task_type.as_str(),
        )
    }
}

pub fn build_stable_key(canonical_url: &str, provider_id: &str, task_type: &str) -> String {
    format!(
        "{}|{}|{}",
        canonical_url.trim().to_ascii_lowercase(),
        provider_id.trim().to_ascii_lowercase(),
        task_type.trim().to_ascii_lowercase()
    )
}

pub fn schedule_deterministically(mut tasks: Vec<RetrievalTask>) -> Vec<RetrievalTask> {
    tasks.sort_by(|left, right| {
        left.priority
            .cmp(&right.priority)
            .then_with(|| left.stable_key().cmp(&right.stable_key()))
            .then_with(|| left.task_id.cmp(&right.task_id))
    });
    tasks
}
