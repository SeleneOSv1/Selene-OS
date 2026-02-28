#![forbid(unsafe_code)]

use selene_kernel_contracts::ph1rem::{Ph1RemRequest, Ph1RemResponse, PH1REM_IMPLEMENTATION_ID};
use selene_kernel_contracts::{ContractViolation, Validate};
use selene_storage::ph1f::{Ph1fStore, StorageError};

pub mod reason_codes {
    use selene_kernel_contracts::ReasonCodeId;

    pub const REM_FAIL_TIME_AMBIGUOUS_NEEDS_CONFIRM: ReasonCodeId = ReasonCodeId(0x5245_00F1);
}

#[derive(Debug, Clone, Default)]
pub struct Ph1RemRuntime;

impl Ph1RemRuntime {
    pub fn run(
        &self,
        store: &mut Ph1fStore,
        req: &Ph1RemRequest,
    ) -> Result<Ph1RemResponse, StorageError> {
        self.run_for_implementation(store, PH1REM_IMPLEMENTATION_ID, req)
    }

    pub fn run_for_implementation(
        &self,
        store: &mut Ph1fStore,
        implementation_id: &str,
        req: &Ph1RemRequest,
    ) -> Result<Ph1RemResponse, StorageError> {
        if implementation_id != PH1REM_IMPLEMENTATION_ID {
            return Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field: "ph1rem.implementation_id",
                    reason: "unknown implementation_id",
                },
            ));
        }

        req.validate().map_err(StorageError::ContractViolation)?;
        store.ph1rem_run(req)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use selene_kernel_contracts::ph1_voice_id::UserId;
    use selene_kernel_contracts::ph1j::{CorrelationId, DeviceId, TurnId};
    use selene_kernel_contracts::ph1position::TenantId;
    use selene_kernel_contracts::ph1rem::{
        ReminderChannel, ReminderDeliverDueCommitRequest, ReminderDeliveryAttemptId,
        ReminderLocalTimeMode, ReminderPriorityLevel, ReminderRequest, ReminderRequestEnvelope,
        ReminderType, REMINDER_DELIVER_DUE_COMMIT,
    };
    use selene_kernel_contracts::MonotonicTimeNs;
    use selene_storage::ph1f::{DeviceRecord, IdentityRecord, IdentityStatus, StorageError};

    fn seeded_store() -> Ph1fStore {
        let mut s = Ph1fStore::new_in_memory();
        let user_id = UserId::new("tenant_demo:user_1").unwrap();
        let device_id = DeviceId::new("device_1").unwrap();
        s.insert_identity(IdentityRecord::v1(
            user_id.clone(),
            None,
            None,
            MonotonicTimeNs(1),
            IdentityStatus::Active,
        ))
        .unwrap();
        s.insert_device(
            DeviceRecord::v1(
                device_id,
                user_id,
                "desktop".to_string(),
                MonotonicTimeNs(1),
                None,
            )
            .unwrap(),
        )
        .unwrap();
        s
    }

    fn schedule_req(idempotency_key: &str, when: &str) -> Ph1RemRequest {
        Ph1RemRequest::schedule_commit_v1(
            CorrelationId(101),
            TurnId(201),
            MonotonicTimeNs(1_000_000_000),
            TenantId::new("tenant_demo").unwrap(),
            UserId::new("tenant_demo:user_1").unwrap(),
            Some(DeviceId::new("device_1").unwrap()),
            ReminderType::Task,
            "call payroll".to_string(),
            when.to_string(),
            "America/Los_Angeles".to_string(),
            ReminderLocalTimeMode::LocalTime,
            ReminderPriorityLevel::Normal,
            None,
            vec![ReminderChannel::Text],
            idempotency_key.to_string(),
        )
        .unwrap()
    }

    #[test]
    fn at_rem_01_schedule_idempotent() {
        let mut store = seeded_store();
        let runtime = Ph1RemRuntime;
        let req = schedule_req("idem_1", "in 15 minutes");

        let out1 = runtime.run(&mut store, &req).unwrap();
        let out2 = runtime.run(&mut store, &req).unwrap();

        let id1 = match out1 {
            Ph1RemResponse::Ok(ok) => ok.reminder_id,
            _ => panic!("expected ok"),
        };
        let id2 = match out2 {
            Ph1RemResponse::Ok(ok) => ok.reminder_id,
            _ => panic!("expected ok"),
        };

        assert_eq!(id1, id2);
    }

    #[test]
    fn at_rem_02_unparseable_time_refused() {
        let mut store = seeded_store();
        let runtime = Ph1RemRuntime;
        let req = schedule_req("idem_2", "later sometime maybe");

        let out = runtime.run(&mut store, &req).unwrap();
        match out {
            Ph1RemResponse::Refuse(r) => {
                assert_eq!(
                    r.reason_code,
                    reason_codes::REM_FAIL_TIME_AMBIGUOUS_NEEDS_CONFIRM
                );
            }
            _ => panic!("expected refusal"),
        }
    }

    #[test]
    fn at_rem_db_01_schedule_round_trip_reads_same_rows() {
        let mut store = seeded_store();
        let runtime = Ph1RemRuntime;
        let req = schedule_req("idem_round_trip", "in 5 minutes");

        let out = runtime.run(&mut store, &req).unwrap();
        let (reminder_id, occurrence_id) = match out {
            Ph1RemResponse::Ok(ok) => (
                ok.reminder_id,
                ok.occurrence_id.expect("schedule must set occurrence_id"),
            ),
            _ => panic!("expected ok"),
        };

        let reminder = store
            .reminder_row(&reminder_id)
            .expect("reminder row must persist");
        let occurrence = store
            .reminder_occurrence_row(&occurrence_id)
            .expect("occurrence row must persist");

        assert_eq!(reminder.reminder_request_text, "call payroll");
        assert_eq!(reminder.reminder_type, ReminderType::Task);
        assert_eq!(occurrence.reminder_id, reminder_id);
    }

    #[test]
    fn at_rem_db_02_delivery_attempts_append_only_guard() {
        let mut store = seeded_store();
        let runtime = Ph1RemRuntime;
        let schedule = schedule_req("idem_delivery_seed", "in 3 minutes");
        let out = runtime.run(&mut store, &schedule).unwrap();
        let (reminder_id, occurrence_id) = match out {
            Ph1RemResponse::Ok(ok) => (
                ok.reminder_id,
                ok.occurrence_id.expect("schedule must set occurrence_id"),
            ),
            _ => panic!("expected ok"),
        };

        let attempt_id = ReminderDeliveryAttemptId::new("attempt_1".to_string()).unwrap();
        let deliver_req = Ph1RemRequest::v1(
            ReminderRequestEnvelope::v1(
                CorrelationId(102),
                TurnId(202),
                MonotonicTimeNs(1_100_000_000),
                REMINDER_DELIVER_DUE_COMMIT.to_string(),
            )
            .unwrap(),
            ReminderRequest::DeliverDueCommit(ReminderDeliverDueCommitRequest {
                tenant_id: TenantId::new("tenant_demo").unwrap(),
                user_id: UserId::new("tenant_demo:user_1").unwrap(),
                reminder_id,
                occurrence_id,
                delivery_channel: ReminderChannel::Text,
                delivery_attempt_id: attempt_id.clone(),
                offline_state: false,
                idempotency_key: "idem_delivery_1".to_string(),
            }),
        )
        .unwrap();

        let _ = runtime.run(&mut store, &deliver_req).unwrap();
        assert_eq!(store.reminder_delivery_attempts().len(), 1);
        assert!(store.reminder_delivery_attempt_row(&attempt_id).is_some());

        let overwrite = store.attempt_overwrite_reminder_delivery_attempt(&attempt_id);
        assert!(matches!(
            overwrite,
            Err(StorageError::AppendOnlyViolation {
                table: "reminder_delivery_attempts"
            })
        ));
    }

    #[test]
    fn at_rem_db_03_delivery_replay_safe_after_index_rebuild() {
        let mut store = seeded_store();
        let runtime = Ph1RemRuntime;
        let schedule = schedule_req("idem_replay_seed", "in 2 minutes");
        let out = runtime.run(&mut store, &schedule).unwrap();
        let (reminder_id, occurrence_id) = match out {
            Ph1RemResponse::Ok(ok) => (
                ok.reminder_id,
                ok.occurrence_id.expect("schedule must set occurrence_id"),
            ),
            _ => panic!("expected ok"),
        };

        let deliver_req = Ph1RemRequest::v1(
            ReminderRequestEnvelope::v1(
                CorrelationId(103),
                TurnId(203),
                MonotonicTimeNs(1_200_000_000),
                REMINDER_DELIVER_DUE_COMMIT.to_string(),
            )
            .unwrap(),
            ReminderRequest::DeliverDueCommit(ReminderDeliverDueCommitRequest {
                tenant_id: TenantId::new("tenant_demo").unwrap(),
                user_id: UserId::new("tenant_demo:user_1").unwrap(),
                reminder_id,
                occurrence_id,
                delivery_channel: ReminderChannel::Text,
                delivery_attempt_id: ReminderDeliveryAttemptId::new("attempt_replay".to_string())
                    .unwrap(),
                offline_state: false,
                idempotency_key: "idem_delivery_replay".to_string(),
            }),
        )
        .unwrap();

        let _ = runtime.run(&mut store, &deliver_req).unwrap();
        assert_eq!(store.reminder_delivery_attempts().len(), 1);

        let mut replayed = store.clone();
        replayed
            .rebuild_reminder_delivery_attempt_indexes()
            .expect("rebuild indexes should succeed");
        let _ = runtime.run(&mut replayed, &deliver_req).unwrap();
        assert_eq!(replayed.reminder_delivery_attempts().len(), 1);
    }
}
