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
        ReminderCancelCommitRequest, ReminderChannel, ReminderDeliverDueCommitRequest,
        ReminderDeliveryAttemptId, ReminderDeliveryRetryScheduleCommitRequest,
        ReminderEscalateCommitRequest, ReminderLocalTimeMode, ReminderOccurrenceId,
        ReminderPriorityLevel, ReminderRequest, ReminderRequestEnvelope,
        ReminderSnoozeCommitRequest, ReminderType, ReminderUpdateCommitRequest,
        ReminderUpdateFields, REMINDER_DELIVERY_RETRY_SCHEDULE_COMMIT, REMINDER_DELIVER_DUE_COMMIT,
        REMINDER_ESCALATE_COMMIT, REMINDER_SNOOZE_COMMIT, REMINDER_UPDATE_COMMIT,
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

    #[allow(clippy::too_many_arguments)]
    fn deliver_due_req(
        correlation_id: u64,
        turn_id: u64,
        now_ns: u64,
        reminder_id: selene_kernel_contracts::ph1rem::ReminderId,
        occurrence_id: ReminderOccurrenceId,
        attempt_id: &str,
        idempotency_key: &str,
        offline_state: bool,
    ) -> Ph1RemRequest {
        Ph1RemRequest::v1(
            ReminderRequestEnvelope::v1(
                CorrelationId(correlation_id as u128),
                TurnId(turn_id),
                MonotonicTimeNs(now_ns),
                REMINDER_DELIVER_DUE_COMMIT.to_string(),
            )
            .unwrap(),
            ReminderRequest::DeliverDueCommit(ReminderDeliverDueCommitRequest {
                tenant_id: TenantId::new("tenant_demo").unwrap(),
                user_id: UserId::new("tenant_demo:user_1").unwrap(),
                reminder_id,
                occurrence_id,
                delivery_channel: ReminderChannel::Text,
                delivery_attempt_id: ReminderDeliveryAttemptId::new(attempt_id.to_string())
                    .unwrap(),
                offline_state,
                idempotency_key: idempotency_key.to_string(),
            }),
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

    #[test]
    fn at_rem_db_04_schedule_with_recurrence_expands_occurrences() {
        let mut store = seeded_store();
        let runtime = Ph1RemRuntime;
        let mut req = schedule_req("idem_recur_daily", "in 1 minute");
        if let ReminderRequest::ScheduleCommit(schedule) = &mut req.request {
            schedule.recurrence_rule = Some("FREQ=DAILY;COUNT=3".to_string());
        } else {
            panic!("expected schedule request");
        }

        let out = runtime.run(&mut store, &req).unwrap();
        let reminder_id = match out {
            Ph1RemResponse::Ok(ok) => ok.reminder_id,
            _ => panic!("expected ok"),
        };

        let mut times: Vec<u64> = store
            .reminder_occurrences()
            .values()
            .filter(|occ| occ.reminder_id == reminder_id)
            .map(|occ| occ.scheduled_time.0)
            .collect();
        times.sort_unstable();
        assert_eq!(times.len(), 3);
        assert_eq!(times[1].saturating_sub(times[0]), 86_400_000_000_000);
        assert_eq!(times[2].saturating_sub(times[1]), 86_400_000_000_000);
    }

    #[test]
    fn at_rem_db_05_update_rebuilds_occurrences_for_recurrence_change() {
        let mut store = seeded_store();
        let runtime = Ph1RemRuntime;
        let schedule = schedule_req("idem_recur_update_seed", "in 5 minutes");
        let out = runtime.run(&mut store, &schedule).unwrap();
        let reminder_id = match out {
            Ph1RemResponse::Ok(ok) => ok.reminder_id,
            _ => panic!("expected ok"),
        };

        let mut fields = ReminderUpdateFields::empty();
        fields.recurrence_rule = Some(Some("FREQ=WEEKLY;COUNT=2".to_string()));
        fields.desired_time = Some("in 10 minutes".to_string());
        let update_req = Ph1RemRequest::v1(
            ReminderRequestEnvelope::v1(
                CorrelationId(104),
                TurnId(204),
                MonotonicTimeNs(1_300_000_000),
                REMINDER_UPDATE_COMMIT.to_string(),
            )
            .unwrap(),
            ReminderRequest::UpdateCommit(ReminderUpdateCommitRequest {
                tenant_id: TenantId::new("tenant_demo").unwrap(),
                user_id: UserId::new("tenant_demo:user_1").unwrap(),
                reminder_id: reminder_id.clone(),
                updated_fields: fields,
                idempotency_key: "idem_recur_update".to_string(),
            }),
        )
        .unwrap();
        let update_out = runtime.run(&mut store, &update_req).unwrap();
        let primary_occurrence_id = match update_out {
            Ph1RemResponse::Ok(ok) => ok.occurrence_id.expect("update should return occurrence"),
            _ => panic!("expected ok"),
        };

        let reminder = store
            .reminder_row(&reminder_id)
            .expect("reminder row should exist");
        assert_eq!(reminder.primary_occurrence_id, primary_occurrence_id);
        let mut times: Vec<u64> = store
            .reminder_occurrences()
            .values()
            .filter(|occ| occ.reminder_id == reminder_id)
            .map(|occ| occ.scheduled_time.0)
            .collect();
        times.sort_unstable();
        assert_eq!(times.len(), 2);
        assert_eq!(times[1].saturating_sub(times[0]), 604_800_000_000_000);
    }

    #[test]
    fn at_rem_db_06_cancel_marks_all_generated_occurrences_canceled() {
        let mut store = seeded_store();
        let runtime = Ph1RemRuntime;
        let mut schedule = schedule_req("idem_cancel_all", "in 1 minute");
        if let ReminderRequest::ScheduleCommit(schedule_req) = &mut schedule.request {
            schedule_req.recurrence_rule = Some("FREQ=DAILY;COUNT=3".to_string());
        } else {
            panic!("expected schedule request");
        }
        let out = runtime.run(&mut store, &schedule).unwrap();
        let reminder_id = match out {
            Ph1RemResponse::Ok(ok) => ok.reminder_id,
            _ => panic!("expected ok"),
        };
        let cancel_req = Ph1RemRequest::v1(
            ReminderRequestEnvelope::v1(
                CorrelationId(105),
                TurnId(205),
                MonotonicTimeNs(1_400_000_000),
                "REMINDER_CANCEL_COMMIT".to_string(),
            )
            .unwrap(),
            ReminderRequest::CancelCommit(ReminderCancelCommitRequest {
                tenant_id: TenantId::new("tenant_demo").unwrap(),
                user_id: UserId::new("tenant_demo:user_1").unwrap(),
                reminder_id: reminder_id.clone(),
                cancel_reason: Some("user request".to_string()),
                idempotency_key: "idem_cancel_all_occurrences".to_string(),
            }),
        )
        .unwrap();
        let _ = runtime.run(&mut store, &cancel_req).unwrap();
        let canceled_count = store
            .reminder_occurrences()
            .values()
            .filter(|occ| occ.reminder_id == reminder_id)
            .filter(|occ| occ.state == selene_kernel_contracts::ph1rem::ReminderState::Canceled)
            .count();
        assert_eq!(canceled_count, 3);
    }

    #[test]
    fn at_rem_db_07_due_fires_once() {
        let mut store = seeded_store();
        let runtime = Ph1RemRuntime;
        let schedule = schedule_req("idem_due_once_seed", "in 1 minute");
        let out = runtime.run(&mut store, &schedule).unwrap();
        let (reminder_id, occurrence_id) = match out {
            Ph1RemResponse::Ok(ok) => (
                ok.reminder_id,
                ok.occurrence_id.expect("schedule must set occurrence_id"),
            ),
            _ => panic!("expected ok"),
        };

        let due_1 = deliver_due_req(
            106,
            206,
            70_000_000_000,
            reminder_id.clone(),
            occurrence_id.clone(),
            "attempt_due_once_1",
            "idem_due_once_1",
            false,
        );
        let due_2 = deliver_due_req(
            107,
            207,
            71_000_000_000,
            reminder_id,
            occurrence_id,
            "attempt_due_once_2",
            "idem_due_once_2",
            false,
        );

        let out_1 = runtime.run(&mut store, &due_1).unwrap();
        let out_2 = runtime.run(&mut store, &due_2).unwrap();
        let attempt_1 = match out_1 {
            Ph1RemResponse::Ok(ok) => ok.delivery_attempt_id.expect("due should emit attempt"),
            _ => panic!("expected ok"),
        };
        let attempt_2 = match out_2 {
            Ph1RemResponse::Ok(ok) => ok.delivery_attempt_id.expect("due should emit attempt"),
            _ => panic!("expected ok"),
        };

        assert_eq!(attempt_1, attempt_2);
        assert_eq!(store.reminder_delivery_attempts().len(), 1);
    }

    #[test]
    fn at_rem_db_08_retry_does_not_double_send() {
        let mut store = seeded_store();
        let runtime = Ph1RemRuntime;
        let schedule = schedule_req("idem_retry_seed", "in 1 minute");
        let out = runtime.run(&mut store, &schedule).unwrap();
        let (reminder_id, occurrence_id) = match out {
            Ph1RemResponse::Ok(ok) => (
                ok.reminder_id,
                ok.occurrence_id.expect("schedule must set occurrence_id"),
            ),
            _ => panic!("expected ok"),
        };

        let due_offline = deliver_due_req(
            108,
            208,
            70_000_000_000,
            reminder_id.clone(),
            occurrence_id.clone(),
            "attempt_retry_seed",
            "idem_retry_seed_delivery",
            true,
        );
        let out_offline = runtime.run(&mut store, &due_offline).unwrap();
        match out_offline {
            Ph1RemResponse::Ok(ok) => {
                assert_eq!(
                    ok.delivery_status,
                    Some(selene_kernel_contracts::ph1rem::ReminderDeliveryStatus::RetryScheduled)
                );
            }
            _ => panic!("expected ok"),
        }

        let retry_req = Ph1RemRequest::v1(
            ReminderRequestEnvelope::v1(
                CorrelationId(109),
                TurnId(209),
                MonotonicTimeNs(75_000_000_000),
                REMINDER_DELIVERY_RETRY_SCHEDULE_COMMIT.to_string(),
            )
            .unwrap(),
            ReminderRequest::DeliveryRetryScheduleCommit(
                ReminderDeliveryRetryScheduleCommitRequest {
                    tenant_id: TenantId::new("tenant_demo").unwrap(),
                    user_id: UserId::new("tenant_demo:user_1").unwrap(),
                    reminder_id: reminder_id.clone(),
                    occurrence_id: occurrence_id.clone(),
                    retry_time: MonotonicTimeNs(80_000_000_000),
                    idempotency_key: "idem_retry_schedule_1".to_string(),
                },
            ),
        )
        .unwrap();
        let _ = runtime.run(&mut store, &retry_req).unwrap();

        let due_retry_1 = deliver_due_req(
            110,
            210,
            81_000_000_000,
            reminder_id.clone(),
            occurrence_id.clone(),
            "attempt_retry_deliver_1",
            "idem_retry_deliver_1",
            false,
        );
        let due_retry_2 = deliver_due_req(
            111,
            211,
            82_000_000_000,
            reminder_id.clone(),
            occurrence_id.clone(),
            "attempt_retry_deliver_2",
            "idem_retry_deliver_2",
            false,
        );
        let out_1 = runtime.run(&mut store, &due_retry_1).unwrap();
        let out_2 = runtime.run(&mut store, &due_retry_2).unwrap();
        let attempt_1 = match out_1 {
            Ph1RemResponse::Ok(ok) => ok.delivery_attempt_id.expect("due should emit attempt"),
            _ => panic!("expected ok"),
        };
        let attempt_2 = match out_2 {
            Ph1RemResponse::Ok(ok) => ok.delivery_attempt_id.expect("due should emit attempt"),
            _ => panic!("expected ok"),
        };

        assert_eq!(attempt_1, attempt_2);
        assert_eq!(store.reminder_delivery_attempts().len(), 2);
    }

    #[test]
    fn at_rem_db_09_escalate_requires_policy() {
        let mut store = seeded_store();
        let runtime = Ph1RemRuntime;
        let schedule = schedule_req("idem_escalate_policy_seed", "in 1 minute");
        let out = runtime.run(&mut store, &schedule).unwrap();
        let (reminder_id, occurrence_id) = match out {
            Ph1RemResponse::Ok(ok) => (
                ok.reminder_id,
                ok.occurrence_id.expect("schedule must set occurrence_id"),
            ),
            _ => panic!("expected ok"),
        };

        let due = deliver_due_req(
            112,
            212,
            70_000_000_000,
            reminder_id.clone(),
            occurrence_id.clone(),
            "attempt_policy_seed",
            "idem_policy_seed_due",
            false,
        );
        let _ = runtime.run(&mut store, &due).unwrap();

        let escalate_req = Ph1RemRequest::v1(
            ReminderRequestEnvelope::v1(
                CorrelationId(113),
                TurnId(213),
                MonotonicTimeNs(71_000_000_000),
                REMINDER_ESCALATE_COMMIT.to_string(),
            )
            .unwrap(),
            ReminderRequest::EscalateCommit(ReminderEscalateCommitRequest {
                tenant_id: TenantId::new("tenant_demo").unwrap(),
                user_id: UserId::new("tenant_demo:user_1").unwrap(),
                reminder_id,
                occurrence_id,
                from_channel: ReminderChannel::Text,
                to_channel: ReminderChannel::Email,
                delivery_attempt_id: ReminderDeliveryAttemptId::new(
                    "attempt_escalate_1".to_string(),
                )
                .unwrap(),
                idempotency_key: "idem_escalate_1".to_string(),
            }),
        )
        .unwrap();
        let out = runtime.run(&mut store, &escalate_req).unwrap();
        match out {
            Ph1RemResponse::Refuse(_) => {}
            _ => panic!("expected escalation policy refusal"),
        }
        assert_eq!(store.reminder_delivery_attempts().len(), 1);
    }

    #[test]
    fn at_rem_db_10_snooze_blocks_due_until_snooze_window_ends() {
        let mut store = seeded_store();
        let runtime = Ph1RemRuntime;
        let schedule = schedule_req("idem_snooze_seed", "in 1 minute");
        let out = runtime.run(&mut store, &schedule).unwrap();
        let (reminder_id, occurrence_id) = match out {
            Ph1RemResponse::Ok(ok) => (
                ok.reminder_id,
                ok.occurrence_id.expect("schedule must set occurrence_id"),
            ),
            _ => panic!("expected ok"),
        };

        let snooze_req = Ph1RemRequest::v1(
            ReminderRequestEnvelope::v1(
                CorrelationId(114),
                TurnId(214),
                MonotonicTimeNs(65_000_000_000),
                REMINDER_SNOOZE_COMMIT.to_string(),
            )
            .unwrap(),
            ReminderRequest::SnoozeCommit(ReminderSnoozeCommitRequest {
                tenant_id: TenantId::new("tenant_demo").unwrap(),
                user_id: UserId::new("tenant_demo:user_1").unwrap(),
                reminder_id: reminder_id.clone(),
                occurrence_id: occurrence_id.clone(),
                snooze_duration_ms: 600_000,
                idempotency_key: "idem_snooze_1".to_string(),
            }),
        )
        .unwrap();
        let _ = runtime.run(&mut store, &snooze_req).unwrap();

        let due_early = deliver_due_req(
            115,
            215,
            66_000_000_000,
            reminder_id.clone(),
            occurrence_id.clone(),
            "attempt_snooze_early",
            "idem_snooze_due_early",
            false,
        );
        let out_early = runtime.run(&mut store, &due_early).unwrap();
        match out_early {
            Ph1RemResponse::Ok(ok) => {
                assert_eq!(
                    ok.delivery_status,
                    Some(selene_kernel_contracts::ph1rem::ReminderDeliveryStatus::RetryScheduled)
                );
            }
            _ => panic!("expected ok"),
        }

        let due_after = deliver_due_req(
            116,
            216,
            670_000_000_000,
            reminder_id.clone(),
            occurrence_id.clone(),
            "attempt_snooze_after",
            "idem_snooze_due_after",
            false,
        );
        let out_after = runtime.run(&mut store, &due_after).unwrap();
        match out_after {
            Ph1RemResponse::Ok(ok) => {
                assert_eq!(
                    ok.delivery_status,
                    Some(selene_kernel_contracts::ph1rem::ReminderDeliveryStatus::Delivered)
                );
            }
            _ => panic!("expected ok"),
        }
    }

    #[test]
    fn at_rem_db_11_missed_due_delivery_sets_followup_pending() {
        let mut store = seeded_store();
        let runtime = Ph1RemRuntime;
        let schedule = schedule_req("idem_missed_due_seed", "in 1 minute");
        let out = runtime.run(&mut store, &schedule).unwrap();
        let (reminder_id, occurrence_id) = match out {
            Ph1RemResponse::Ok(ok) => (
                ok.reminder_id,
                ok.occurrence_id.expect("schedule must set occurrence_id"),
            ),
            _ => panic!("expected ok"),
        };

        let due_missed = deliver_due_req(
            117,
            217,
            190_000_000_000,
            reminder_id.clone(),
            occurrence_id.clone(),
            "attempt_missed_due_1",
            "idem_missed_due_1",
            false,
        );
        let _ = runtime.run(&mut store, &due_missed).unwrap();

        let occurrence = store
            .reminder_occurrence_row(&occurrence_id)
            .expect("occurrence row should exist");
        assert_eq!(
            occurrence.state,
            selene_kernel_contracts::ph1rem::ReminderState::FollowupPending
        );
        assert_eq!(
            occurrence.followup_time,
            Some(MonotonicTimeNs(490_000_000_000))
        );
    }
}
