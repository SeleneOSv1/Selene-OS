#![forbid(unsafe_code)]

use crate::web_search_plan::learning::promotion_gate::PolicySnapshot;

pub const ROLLBACK_VERSION: &str = "1.0.0";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RollbackRecord {
    pub from_policy_snapshot_version: String,
    pub restored_policy_snapshot_version: String,
    pub rollback_at_ms: i64,
    pub rollback_status: String,
    pub rollback_version: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RollbackError {
    PriorSnapshotMissing,
}

#[derive(Debug, Clone)]
pub struct PolicyHistory {
    current: PolicySnapshot,
    prior: Option<PolicySnapshot>,
    rollback_events: Vec<RollbackRecord>,
}

impl PolicyHistory {
    pub fn new(initial: PolicySnapshot) -> Self {
        Self {
            current: initial,
            prior: None,
            rollback_events: Vec::new(),
        }
    }

    pub fn current(&self) -> &PolicySnapshot {
        &self.current
    }

    pub fn prior(&self) -> Option<&PolicySnapshot> {
        self.prior.as_ref()
    }

    pub fn rollback_events(&self) -> &[RollbackRecord] {
        &self.rollback_events
    }

    pub fn promote(&mut self, promoted_snapshot: PolicySnapshot) {
        self.prior = Some(self.current.clone());
        self.current = promoted_snapshot;
    }

    pub fn rollback(&mut self, rollback_at_ms: i64) -> Result<RollbackRecord, RollbackError> {
        let prior = self
            .prior
            .clone()
            .ok_or(RollbackError::PriorSnapshotMissing)?;

        let record = RollbackRecord {
            from_policy_snapshot_version: self.current.policy_snapshot_version.clone(),
            restored_policy_snapshot_version: prior.policy_snapshot_version.clone(),
            rollback_at_ms,
            rollback_status: "rolled_back".to_string(),
            rollback_version: ROLLBACK_VERSION.to_string(),
        };

        self.current = prior;
        self.prior = None;
        self.rollback_events.push(record.clone());

        Ok(record)
    }
}
