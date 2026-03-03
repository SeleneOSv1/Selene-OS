#![forbid(unsafe_code)]

use crate::web_search_plan::learn::promotion_gate::{PolicySnapshotReference, PromotionRecord};

pub const ROLLBACK_RECORD_VERSION: &str = "1.0.0";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RollbackRecord {
    pub from_policy_snapshot_version: String,
    pub restored_policy_snapshot_version: String,
    pub rollback_at_ms: i64,
    pub rollback_requested_by: String,
    pub rollback_status: String,
    pub version: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RollbackError {
    PriorSnapshotMissing,
}

#[derive(Debug, Clone)]
pub struct PromotionState {
    current: PolicySnapshotReference,
    previous: Option<PolicySnapshotReference>,
    promotion_records: Vec<PromotionRecord>,
    rollback_records: Vec<RollbackRecord>,
}

impl PromotionState {
    pub fn new(initial_policy: PolicySnapshotReference) -> Self {
        Self {
            current: initial_policy,
            previous: None,
            promotion_records: Vec::new(),
            rollback_records: Vec::new(),
        }
    }

    pub fn current(&self) -> &PolicySnapshotReference {
        &self.current
    }

    pub fn previous(&self) -> Option<&PolicySnapshotReference> {
        self.previous.as_ref()
    }

    pub fn promotion_records(&self) -> &[PromotionRecord] {
        &self.promotion_records
    }

    pub fn rollback_records(&self) -> &[RollbackRecord] {
        &self.rollback_records
    }

    pub fn apply_promotion(
        &mut self,
        promotion_record: PromotionRecord,
        promoted_policy: PolicySnapshotReference,
    ) {
        self.previous = Some(self.current.clone());
        self.current = promoted_policy;
        self.promotion_records.push(promotion_record);
    }

    pub fn rollback(
        &mut self,
        rollback_at_ms: i64,
        rollback_requested_by: &str,
    ) -> Result<RollbackRecord, RollbackError> {
        let previous = self
            .previous
            .clone()
            .ok_or(RollbackError::PriorSnapshotMissing)?;

        let record = RollbackRecord {
            from_policy_snapshot_version: self.current.policy_snapshot_version.clone(),
            restored_policy_snapshot_version: previous.policy_snapshot_version.clone(),
            rollback_at_ms,
            rollback_requested_by: rollback_requested_by.trim().to_string(),
            rollback_status: "rolled_back".to_string(),
            version: ROLLBACK_RECORD_VERSION.to_string(),
        };

        self.current = previous;
        self.previous = None;
        self.rollback_records.push(record.clone());
        Ok(record)
    }
}
