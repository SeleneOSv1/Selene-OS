#![forbid(unsafe_code)]

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use selene_kernel_contracts::ph1rem::{
    Ph1RemOk, Ph1RemRefuse, Ph1RemRequest, Ph1RemResponse, ReminderChannel,
    ReminderDeliveryAttemptId, ReminderDeliveryStatus, ReminderId, ReminderLocalTimeMode,
    ReminderOccurrenceId, ReminderPriorityLevel, ReminderRequest, ReminderState, ReminderType,
    PH1REM_IMPLEMENTATION_ID,
};
use selene_kernel_contracts::{ContractViolation, MonotonicTimeNs, ReasonCodeId, Validate};
use selene_storage::ph1f::{Ph1fStore, StorageError};

pub mod reason_codes {
    use selene_kernel_contracts::ReasonCodeId;

    pub const REMINDER_SCHEDULED: ReasonCodeId = ReasonCodeId(0x5245_0001);
    pub const REMINDER_UPDATED: ReasonCodeId = ReasonCodeId(0x5245_0002);
    pub const REMINDER_CANCELED: ReasonCodeId = ReasonCodeId(0x5245_0003);
    pub const REMINDER_SNOOZED: ReasonCodeId = ReasonCodeId(0x5245_0004);
    pub const REMINDER_FOLLOWUP_SCHEDULED: ReasonCodeId = ReasonCodeId(0x5245_0005);
    pub const REMINDER_RETRY_SCHEDULED: ReasonCodeId = ReasonCodeId(0x5245_0006);
    pub const REMINDER_DELIVERED_PRE: ReasonCodeId = ReasonCodeId(0x5245_0007);
    pub const REMINDER_DELIVERED_DUE: ReasonCodeId = ReasonCodeId(0x5245_0008);
    pub const REMINDER_ESCALATED: ReasonCodeId = ReasonCodeId(0x5245_0009);
    pub const REMINDER_COMPLETED: ReasonCodeId = ReasonCodeId(0x5245_000A);
    pub const REMINDER_FAILED: ReasonCodeId = ReasonCodeId(0x5245_000B);

    pub const REM_FAIL_TIME_AMBIGUOUS_NEEDS_CONFIRM: ReasonCodeId = ReasonCodeId(0x5245_00F1);
    pub const REM_FAIL_NOT_FOUND: ReasonCodeId = ReasonCodeId(0x5245_00F2);
    pub const REM_FAIL_SCOPE_VIOLATION: ReasonCodeId = ReasonCodeId(0x5245_00F3);
    pub const REM_FAIL_STATE_TRANSITION_INVALID: ReasonCodeId = ReasonCodeId(0x5245_00F4);
    pub const REM_FAIL_INTERNAL: ReasonCodeId = ReasonCodeId(0x5245_00FF);
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
struct ReminderRow {
    reminder_id: ReminderId,
    tenant_id: selene_kernel_contracts::ph1position::TenantId,
    user_id: selene_kernel_contracts::ph1_voice_id::UserId,
    device_id: Option<selene_kernel_contracts::ph1j::DeviceId>,
    reminder_type: ReminderType,
    reminder_request_text: String,
    desired_time_text: String,
    resolved_due_at: MonotonicTimeNs,
    user_timezone: String,
    local_time_mode: ReminderLocalTimeMode,
    priority_level: ReminderPriorityLevel,
    recurrence_rule: Option<String>,
    channel_preferences: Vec<ReminderChannel>,
    state: ReminderState,
    primary_occurrence_id: ReminderOccurrenceId,
    created_at: MonotonicTimeNs,
    updated_at: MonotonicTimeNs,
}

#[derive(Debug, Clone)]
struct ReminderOccurrenceRow {
    occurrence_id: ReminderOccurrenceId,
    reminder_id: ReminderId,
    scheduled_time: MonotonicTimeNs,
    state: ReminderState,
    snooze_until: Option<MonotonicTimeNs>,
    followup_time: Option<MonotonicTimeNs>,
    retry_time: Option<MonotonicTimeNs>,
    completed_at: Option<MonotonicTimeNs>,
    failure_reason: Option<String>,
    updated_at: MonotonicTimeNs,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
struct ReminderDeliveryAttemptRow {
    attempt_id: ReminderDeliveryAttemptId,
    reminder_id: ReminderId,
    occurrence_id: ReminderOccurrenceId,
    channel: ReminderChannel,
    status: ReminderDeliveryStatus,
    proof_ref: Option<String>,
    created_at: MonotonicTimeNs,
}

#[derive(Debug, Default)]
struct Ph1RemState {
    next_reminder_seq: u64,
    next_occurrence_seq: u64,
    reminders: BTreeMap<ReminderId, ReminderRow>,
    occurrences: BTreeMap<ReminderOccurrenceId, ReminderOccurrenceRow>,
    delivery_attempts: Vec<ReminderDeliveryAttemptRow>,
    schedule_idempotency: BTreeMap<
        (
            selene_kernel_contracts::ph1position::TenantId,
            selene_kernel_contracts::ph1_voice_id::UserId,
            MonotonicTimeNs,
            ReminderType,
            String,
        ),
        ReminderId,
    >,
    update_idempotency: BTreeMap<
        (
            selene_kernel_contracts::ph1position::TenantId,
            ReminderId,
            String,
            &'static str,
        ),
        ReminderId,
    >,
    delivery_idempotency: BTreeMap<
        (
            selene_kernel_contracts::ph1position::TenantId,
            ReminderId,
            ReminderOccurrenceId,
            ReminderDeliveryAttemptId,
            &'static str,
        ),
        usize,
    >,
}

#[derive(Debug, Clone, Default)]
pub struct Ph1RemRuntime {
    state: Arc<Mutex<Ph1RemState>>,
}

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

        match &req.request {
            ReminderRequest::ScheduleCommit(r) => self.run_schedule(store, req, r),
            ReminderRequest::UpdateCommit(r) => self.run_update(req, r),
            ReminderRequest::CancelCommit(r) => self.run_cancel(req, r),
            ReminderRequest::SnoozeCommit(r) => self.run_snooze(req, r),
            ReminderRequest::FollowupScheduleCommit(r) => self.run_followup(req, r),
            ReminderRequest::DeliveryRetryScheduleCommit(r) => self.run_retry(req, r),
            ReminderRequest::DeliverPreCommit(r) => {
                self.run_delivery(req, r, false, ReminderDeliveryStatus::Delivered)
            }
            ReminderRequest::DeliverDueCommit(r) => {
                let status = if r.offline_state {
                    ReminderDeliveryStatus::RetryScheduled
                } else {
                    ReminderDeliveryStatus::Delivered
                };
                self.run_delivery_due(req, r, status)
            }
            ReminderRequest::EscalateCommit(r) => self.run_escalate(req, r),
            ReminderRequest::MarkCompletedCommit(r) => self.run_mark_completed(req, r),
            ReminderRequest::MarkFailedCommit(r) => self.run_mark_failed(req, r),
        }
    }

    fn run_schedule(
        &self,
        store: &mut Ph1fStore,
        req: &Ph1RemRequest,
        r: &selene_kernel_contracts::ph1rem::ReminderScheduleCommitRequest,
    ) -> Result<Ph1RemResponse, StorageError> {
        if store.get_identity(&r.user_id).is_none() {
            return Ok(Ph1RemResponse::Refuse(Ph1RemRefuse::v1(
                req.envelope.simulation_id.clone(),
                reason_codes::REM_FAIL_SCOPE_VIOLATION,
                "identity not found for user_id".to_string(),
            )?));
        }

        if let Some(device_id) = &r.device_id {
            if store.get_device(device_id).is_none() {
                return Ok(Ph1RemResponse::Refuse(Ph1RemRefuse::v1(
                    req.envelope.simulation_id.clone(),
                    reason_codes::REM_FAIL_SCOPE_VIOLATION,
                    "device not found for device_id".to_string(),
                )?));
            }
        }

        let resolved_due_at = match resolve_desired_time(req.envelope.now, &r.desired_time) {
            Some(t) => t,
            None => {
                return Ok(Ph1RemResponse::Refuse(Ph1RemRefuse::v1(
                    req.envelope.simulation_id.clone(),
                    reason_codes::REM_FAIL_TIME_AMBIGUOUS_NEEDS_CONFIRM,
                    "desired_time is not deterministically parseable".to_string(),
                )?))
            }
        };

        let mut state = self.lock_state()?;
        let idx = (
            r.tenant_id.clone(),
            r.user_id.clone(),
            resolved_due_at,
            r.reminder_type,
            r.idempotency_key.clone(),
        );

        if let Some(existing_id) = state.schedule_idempotency.get(&idx).cloned() {
            let row = state
                .reminders
                .get(&existing_id)
                .ok_or(StorageError::ContractViolation(
                    ContractViolation::InvalidValue {
                        field: "ph1rem.schedule.idempotency",
                        reason: "existing reminder id missing",
                    },
                ))?;

            return Ok(Ph1RemResponse::Ok(Ph1RemOk::v1(
                req.envelope.simulation_id.clone(),
                reason_codes::REMINDER_SCHEDULED,
                row.reminder_id.clone(),
                Some(row.primary_occurrence_id.clone()),
                row.state,
                Some(row.resolved_due_at),
                None,
                None,
                None,
                None,
            )?));
        }

        state.next_reminder_seq = state.next_reminder_seq.saturating_add(1);
        state.next_occurrence_seq = state.next_occurrence_seq.saturating_add(1);

        let reminder_id = ReminderId::new(format!("rem_{:016x}", state.next_reminder_seq))?;
        let occurrence_id =
            ReminderOccurrenceId::new(format!("occ_{:016x}", state.next_occurrence_seq))?;

        let row = ReminderRow {
            reminder_id: reminder_id.clone(),
            tenant_id: r.tenant_id.clone(),
            user_id: r.user_id.clone(),
            device_id: r.device_id.clone(),
            reminder_type: r.reminder_type,
            reminder_request_text: r.reminder_request_text.clone(),
            desired_time_text: r.desired_time.clone(),
            resolved_due_at,
            user_timezone: r.user_timezone.clone(),
            local_time_mode: r.local_time_mode,
            priority_level: r.priority_level,
            recurrence_rule: r.recurrence_rule.clone(),
            channel_preferences: r.channel_preferences.clone(),
            state: ReminderState::Scheduled,
            primary_occurrence_id: occurrence_id.clone(),
            created_at: req.envelope.now,
            updated_at: req.envelope.now,
        };

        let occ = ReminderOccurrenceRow {
            occurrence_id: occurrence_id.clone(),
            reminder_id: reminder_id.clone(),
            scheduled_time: resolved_due_at,
            state: ReminderState::Scheduled,
            snooze_until: None,
            followup_time: None,
            retry_time: None,
            completed_at: None,
            failure_reason: None,
            updated_at: req.envelope.now,
        };

        state.reminders.insert(reminder_id.clone(), row);
        state.occurrences.insert(occurrence_id.clone(), occ);
        state.schedule_idempotency.insert(idx, reminder_id.clone());

        Ok(Ph1RemResponse::Ok(Ph1RemOk::v1(
            req.envelope.simulation_id.clone(),
            reason_codes::REMINDER_SCHEDULED,
            reminder_id,
            Some(occurrence_id),
            ReminderState::Scheduled,
            Some(resolved_due_at),
            None,
            None,
            None,
            None,
        )?))
    }

    fn run_update(
        &self,
        req: &Ph1RemRequest,
        r: &selene_kernel_contracts::ph1rem::ReminderUpdateCommitRequest,
    ) -> Result<Ph1RemResponse, StorageError> {
        let mut state = self.lock_state()?;
        let idx = (
            r.tenant_id.clone(),
            r.reminder_id.clone(),
            r.idempotency_key.clone(),
            "UPDATE",
        );
        if let Some(existing_id) = state.update_idempotency.get(&idx).cloned() {
            let row = state
                .reminders
                .get(&existing_id)
                .ok_or(StorageError::ContractViolation(
                    ContractViolation::InvalidValue {
                        field: "ph1rem.update.idempotency",
                        reason: "existing reminder id missing",
                    },
                ))?;
            return Ok(Ph1RemResponse::Ok(Ph1RemOk::v1(
                req.envelope.simulation_id.clone(),
                reason_codes::REMINDER_UPDATED,
                row.reminder_id.clone(),
                Some(row.primary_occurrence_id.clone()),
                row.state,
                Some(row.resolved_due_at),
                None,
                None,
                None,
                None,
            )?));
        }

        let parsed_due = if let Some(v) = &r.updated_fields.desired_time {
            match resolve_desired_time(req.envelope.now, v) {
                Some(parsed) => Some((v.clone(), parsed)),
                None => {
                    return Ok(Ph1RemResponse::Refuse(Ph1RemRefuse::v1(
                        req.envelope.simulation_id.clone(),
                        reason_codes::REM_FAIL_TIME_AMBIGUOUS_NEEDS_CONFIRM,
                        "desired_time is not deterministically parseable".to_string(),
                    )?))
                }
            }
        } else {
            None
        };

        let (reminder_id, primary_occurrence_id, state_after, resolved_due_at) = {
            let row = state.reminders.get_mut(&r.reminder_id).ok_or_else(|| {
                StorageError::ContractViolation(ContractViolation::InvalidValue {
                    field: "ph1rem.update.reminder_id",
                    reason: "not found",
                })
            })?;

            if row.tenant_id != r.tenant_id || row.user_id != r.user_id {
                return Ok(Ph1RemResponse::Refuse(Ph1RemRefuse::v1(
                    req.envelope.simulation_id.clone(),
                    reason_codes::REM_FAIL_SCOPE_VIOLATION,
                    "tenant_id/user_id do not match reminder owner".to_string(),
                )?));
            }

            if row.state == ReminderState::Canceled
                || row.state == ReminderState::Completed
                || row.state == ReminderState::Failed
            {
                return Ok(Ph1RemResponse::Refuse(Ph1RemRefuse::v1(
                    req.envelope.simulation_id.clone(),
                    reason_codes::REM_FAIL_STATE_TRANSITION_INVALID,
                    "cannot update terminal reminder state".to_string(),
                )?));
            }

            if let Some(v) = &r.updated_fields.reminder_request_text {
                row.reminder_request_text = v.clone();
            }
            if let Some(v) = &r.updated_fields.user_timezone {
                row.user_timezone = v.clone();
            }
            if let Some(v) = r.updated_fields.local_time_mode {
                row.local_time_mode = v;
            }
            if let Some(v) = r.updated_fields.priority_level {
                row.priority_level = v;
            }
            if let Some(v) = &r.updated_fields.recurrence_rule {
                row.recurrence_rule = v.clone();
            }
            if let Some(v) = &r.updated_fields.channel_preferences {
                row.channel_preferences = v.clone();
            }
            if let Some((desired_time_text, parsed)) = &parsed_due {
                row.desired_time_text = desired_time_text.clone();
                row.resolved_due_at = *parsed;
            }
            row.updated_at = req.envelope.now;

            (
                row.reminder_id.clone(),
                row.primary_occurrence_id.clone(),
                row.state,
                row.resolved_due_at,
            )
        };

        if let Some((_desired_time_text, parsed)) = parsed_due {
            if let Some(occ) = state.occurrences.get_mut(&primary_occurrence_id) {
                occ.scheduled_time = parsed;
                occ.updated_at = req.envelope.now;
            }
        }

        state.update_idempotency.insert(idx, reminder_id.clone());

        Ok(Ph1RemResponse::Ok(Ph1RemOk::v1(
            req.envelope.simulation_id.clone(),
            reason_codes::REMINDER_UPDATED,
            reminder_id,
            Some(primary_occurrence_id),
            state_after,
            Some(resolved_due_at),
            None,
            None,
            None,
            None,
        )?))
    }

    fn run_cancel(
        &self,
        req: &Ph1RemRequest,
        r: &selene_kernel_contracts::ph1rem::ReminderCancelCommitRequest,
    ) -> Result<Ph1RemResponse, StorageError> {
        let mut state = self.lock_state()?;
        let idx = (
            r.tenant_id.clone(),
            r.reminder_id.clone(),
            r.idempotency_key.clone(),
            "CANCEL",
        );
        if let Some(existing_id) = state.update_idempotency.get(&idx).cloned() {
            let row = state
                .reminders
                .get(&existing_id)
                .ok_or(StorageError::ContractViolation(
                    ContractViolation::InvalidValue {
                        field: "ph1rem.cancel.idempotency",
                        reason: "existing reminder id missing",
                    },
                ))?;
            return Ok(Ph1RemResponse::Ok(Ph1RemOk::v1(
                req.envelope.simulation_id.clone(),
                reason_codes::REMINDER_CANCELED,
                row.reminder_id.clone(),
                Some(row.primary_occurrence_id.clone()),
                row.state,
                Some(row.resolved_due_at),
                None,
                None,
                None,
                None,
            )?));
        }

        let (reminder_id, primary_occurrence_id, state_after, resolved_due_at) = {
            let row = state.reminders.get_mut(&r.reminder_id).ok_or_else(|| {
                StorageError::ContractViolation(ContractViolation::InvalidValue {
                    field: "ph1rem.cancel.reminder_id",
                    reason: "not found",
                })
            })?;

            if row.tenant_id != r.tenant_id || row.user_id != r.user_id {
                return Ok(Ph1RemResponse::Refuse(Ph1RemRefuse::v1(
                    req.envelope.simulation_id.clone(),
                    reason_codes::REM_FAIL_SCOPE_VIOLATION,
                    "tenant_id/user_id do not match reminder owner".to_string(),
                )?));
            }

            row.state = ReminderState::Canceled;
            row.updated_at = req.envelope.now;
            (
                row.reminder_id.clone(),
                row.primary_occurrence_id.clone(),
                row.state,
                row.resolved_due_at,
            )
        };

        for occ in state.occurrences.values_mut() {
            if occ.reminder_id == reminder_id
                && occ.state != ReminderState::Completed
                && occ.state != ReminderState::Failed
            {
                occ.state = ReminderState::Canceled;
                occ.updated_at = req.envelope.now;
            }
        }

        state.update_idempotency.insert(idx, reminder_id.clone());

        Ok(Ph1RemResponse::Ok(Ph1RemOk::v1(
            req.envelope.simulation_id.clone(),
            reason_codes::REMINDER_CANCELED,
            reminder_id,
            Some(primary_occurrence_id),
            state_after,
            Some(resolved_due_at),
            None,
            None,
            None,
            None,
        )?))
    }

    fn run_snooze(
        &self,
        req: &Ph1RemRequest,
        r: &selene_kernel_contracts::ph1rem::ReminderSnoozeCommitRequest,
    ) -> Result<Ph1RemResponse, StorageError> {
        self.run_occurrence_mutation(
            req,
            &r.tenant_id,
            &r.user_id,
            &r.reminder_id,
            &r.occurrence_id,
            &r.idempotency_key,
            "SNOOZE",
            |row, occ, now| {
                let snooze_until = MonotonicTimeNs(
                    now.0
                        .saturating_add((r.snooze_duration_ms as u64) * 1_000_000),
                );
                row.state = ReminderState::Snoozed;
                row.updated_at = now;
                occ.state = ReminderState::Snoozed;
                occ.scheduled_time = snooze_until;
                occ.snooze_until = Some(snooze_until);
                occ.updated_at = now;
                Ok((
                    reason_codes::REMINDER_SNOOZED,
                    ReminderState::Snoozed,
                    Some(occ.scheduled_time),
                ))
            },
        )
    }

    fn run_followup(
        &self,
        req: &Ph1RemRequest,
        r: &selene_kernel_contracts::ph1rem::ReminderFollowupScheduleCommitRequest,
    ) -> Result<Ph1RemResponse, StorageError> {
        self.run_occurrence_mutation(
            req,
            &r.tenant_id,
            &r.user_id,
            &r.reminder_id,
            &r.occurrence_id,
            &r.idempotency_key,
            "FOLLOWUP",
            |row, occ, now| {
                let followup_time = MonotonicTimeNs(
                    now.0
                        .saturating_add((r.followup_delay_ms as u64) * 1_000_000),
                );
                row.state = ReminderState::FollowupPending;
                row.updated_at = now;
                occ.state = ReminderState::FollowupPending;
                occ.followup_time = Some(followup_time);
                occ.updated_at = now;
                Ok((
                    reason_codes::REMINDER_FOLLOWUP_SCHEDULED,
                    ReminderState::FollowupPending,
                    Some(followup_time),
                ))
            },
        )
    }

    fn run_retry(
        &self,
        req: &Ph1RemRequest,
        r: &selene_kernel_contracts::ph1rem::ReminderDeliveryRetryScheduleCommitRequest,
    ) -> Result<Ph1RemResponse, StorageError> {
        self.run_occurrence_mutation(
            req,
            &r.tenant_id,
            &r.user_id,
            &r.reminder_id,
            &r.occurrence_id,
            &r.idempotency_key,
            "RETRY",
            |row, occ, now| {
                row.updated_at = now;
                occ.retry_time = Some(r.retry_time);
                occ.updated_at = now;
                Ok((
                    reason_codes::REMINDER_RETRY_SCHEDULED,
                    row.state,
                    Some(r.retry_time),
                ))
            },
        )
    }

    fn run_delivery(
        &self,
        req: &Ph1RemRequest,
        r: &selene_kernel_contracts::ph1rem::ReminderDeliverPreCommitRequest,
        _due: bool,
        status: ReminderDeliveryStatus,
    ) -> Result<Ph1RemResponse, StorageError> {
        self.run_delivery_common(
            req,
            &r.tenant_id,
            &r.user_id,
            &r.reminder_id,
            &r.occurrence_id,
            r.delivery_channel,
            &r.delivery_attempt_id,
            &r.idempotency_key,
            "DELIVER_PRE",
            status,
            reason_codes::REMINDER_DELIVERED_PRE,
            None,
        )
    }

    fn run_delivery_due(
        &self,
        req: &Ph1RemRequest,
        r: &selene_kernel_contracts::ph1rem::ReminderDeliverDueCommitRequest,
        status: ReminderDeliveryStatus,
    ) -> Result<Ph1RemResponse, StorageError> {
        self.run_delivery_common(
            req,
            &r.tenant_id,
            &r.user_id,
            &r.reminder_id,
            &r.occurrence_id,
            r.delivery_channel,
            &r.delivery_attempt_id,
            &r.idempotency_key,
            "DELIVER_DUE",
            status,
            reason_codes::REMINDER_DELIVERED_DUE,
            None,
        )
    }

    fn run_escalate(
        &self,
        req: &Ph1RemRequest,
        r: &selene_kernel_contracts::ph1rem::ReminderEscalateCommitRequest,
    ) -> Result<Ph1RemResponse, StorageError> {
        if r.from_channel == r.to_channel {
            return Ok(Ph1RemResponse::Refuse(Ph1RemRefuse::v1(
                req.envelope.simulation_id.clone(),
                reason_codes::REM_FAIL_STATE_TRANSITION_INVALID,
                "from_channel and to_channel must differ".to_string(),
            )?));
        }

        self.run_delivery_common(
            req,
            &r.tenant_id,
            &r.user_id,
            &r.reminder_id,
            &r.occurrence_id,
            r.to_channel,
            &r.delivery_attempt_id,
            &r.idempotency_key,
            "ESCALATE",
            ReminderDeliveryStatus::Delivered,
            reason_codes::REMINDER_ESCALATED,
            Some(1),
        )
    }

    fn run_mark_completed(
        &self,
        req: &Ph1RemRequest,
        r: &selene_kernel_contracts::ph1rem::ReminderMarkCompletedCommitRequest,
    ) -> Result<Ph1RemResponse, StorageError> {
        let _ = r.ack_source;
        self.run_occurrence_mutation(
            req,
            &r.tenant_id,
            &r.user_id,
            &r.reminder_id,
            &r.occurrence_id,
            &r.idempotency_key,
            "COMPLETE",
            |row, occ, now| {
                row.state = ReminderState::Completed;
                row.updated_at = now;
                occ.state = ReminderState::Completed;
                occ.completed_at = Some(now);
                occ.updated_at = now;
                Ok((
                    reason_codes::REMINDER_COMPLETED,
                    ReminderState::Completed,
                    Some(occ.scheduled_time),
                ))
            },
        )
    }

    fn run_mark_failed(
        &self,
        req: &Ph1RemRequest,
        r: &selene_kernel_contracts::ph1rem::ReminderMarkFailedCommitRequest,
    ) -> Result<Ph1RemResponse, StorageError> {
        self.run_occurrence_mutation(
            req,
            &r.tenant_id,
            &r.user_id,
            &r.reminder_id,
            &r.occurrence_id,
            &r.idempotency_key,
            "FAIL",
            |row, occ, now| {
                row.state = ReminderState::Failed;
                row.updated_at = now;
                occ.state = ReminderState::Failed;
                occ.failure_reason = Some(r.failure_reason.clone());
                occ.updated_at = now;
                Ok((
                    reason_codes::REMINDER_FAILED,
                    ReminderState::Failed,
                    Some(occ.scheduled_time),
                ))
            },
        )
    }

    fn run_occurrence_mutation<F>(
        &self,
        req: &Ph1RemRequest,
        tenant_id: &selene_kernel_contracts::ph1position::TenantId,
        user_id: &selene_kernel_contracts::ph1_voice_id::UserId,
        reminder_id: &ReminderId,
        occurrence_id: &ReminderOccurrenceId,
        idempotency_key: &str,
        action: &'static str,
        mutator: F,
    ) -> Result<Ph1RemResponse, StorageError>
    where
        F: FnOnce(
            &mut ReminderRow,
            &mut ReminderOccurrenceRow,
            MonotonicTimeNs,
        )
            -> Result<(ReasonCodeId, ReminderState, Option<MonotonicTimeNs>), StorageError>,
    {
        let mut state = self.lock_state()?;
        let idx = (
            tenant_id.clone(),
            reminder_id.clone(),
            idempotency_key.to_string(),
            action,
        );
        if let Some(existing_id) = state.update_idempotency.get(&idx).cloned() {
            let row = state
                .reminders
                .get(&existing_id)
                .ok_or(StorageError::ContractViolation(
                    ContractViolation::InvalidValue {
                        field: "ph1rem.occurrence.idempotency",
                        reason: "existing reminder id missing",
                    },
                ))?;
            return Ok(Ph1RemResponse::Ok(Ph1RemOk::v1(
                req.envelope.simulation_id.clone(),
                reason_codes::REMINDER_UPDATED,
                row.reminder_id.clone(),
                Some(row.primary_occurrence_id.clone()),
                row.state,
                Some(row.resolved_due_at),
                None,
                None,
                None,
                None,
            )?));
        }

        let row_snapshot = state
            .reminders
            .get(reminder_id)
            .ok_or_else(|| {
                StorageError::ContractViolation(ContractViolation::InvalidValue {
                    field: "ph1rem.reminder_id",
                    reason: "not found",
                })
            })?
            .clone();
        if row_snapshot.tenant_id != *tenant_id || row_snapshot.user_id != *user_id {
            return Ok(Ph1RemResponse::Refuse(Ph1RemRefuse::v1(
                req.envelope.simulation_id.clone(),
                reason_codes::REM_FAIL_SCOPE_VIOLATION,
                "tenant_id/user_id do not match reminder owner".to_string(),
            )?));
        }

        let occ_snapshot = state
            .occurrences
            .get(occurrence_id)
            .ok_or_else(|| {
                StorageError::ContractViolation(ContractViolation::InvalidValue {
                    field: "ph1rem.occurrence_id",
                    reason: "not found",
                })
            })?
            .clone();

        if occ_snapshot.reminder_id != row_snapshot.reminder_id {
            return Ok(Ph1RemResponse::Refuse(Ph1RemRefuse::v1(
                req.envelope.simulation_id.clone(),
                reason_codes::REM_FAIL_SCOPE_VIOLATION,
                "occurrence does not belong to reminder_id".to_string(),
            )?));
        }

        if occ_snapshot.state == ReminderState::Canceled
            || occ_snapshot.state == ReminderState::Completed
            || occ_snapshot.state == ReminderState::Failed
        {
            return Ok(Ph1RemResponse::Refuse(Ph1RemRefuse::v1(
                req.envelope.simulation_id.clone(),
                reason_codes::REM_FAIL_STATE_TRANSITION_INVALID,
                "cannot mutate terminal occurrence state".to_string(),
            )?));
        }

        let mut row = state.reminders.remove(reminder_id).ok_or_else(|| {
            StorageError::ContractViolation(ContractViolation::InvalidValue {
                field: "ph1rem.reminder_id",
                reason: "not found",
            })
        })?;
        let mut occ = state.occurrences.remove(occurrence_id).ok_or_else(|| {
            StorageError::ContractViolation(ContractViolation::InvalidValue {
                field: "ph1rem.occurrence_id",
                reason: "not found",
            })
        })?;

        let (reason_code, state_after, scheduled_time) =
            mutator(&mut row, &mut occ, req.envelope.now)?;
        let out_reminder_id = row.reminder_id.clone();
        let out_occurrence_id = occ.occurrence_id.clone();

        state.reminders.insert(reminder_id.clone(), row);
        state.occurrences.insert(occurrence_id.clone(), occ);
        state
            .update_idempotency
            .insert(idx, out_reminder_id.clone());

        Ok(Ph1RemResponse::Ok(Ph1RemOk::v1(
            req.envelope.simulation_id.clone(),
            reason_code,
            out_reminder_id,
            Some(out_occurrence_id),
            state_after,
            scheduled_time,
            None,
            None,
            None,
            None,
        )?))
    }

    #[allow(clippy::too_many_arguments)]
    fn run_delivery_common(
        &self,
        req: &Ph1RemRequest,
        tenant_id: &selene_kernel_contracts::ph1position::TenantId,
        user_id: &selene_kernel_contracts::ph1_voice_id::UserId,
        reminder_id: &ReminderId,
        occurrence_id: &ReminderOccurrenceId,
        channel: ReminderChannel,
        delivery_attempt_id: &ReminderDeliveryAttemptId,
        idempotency_key: &str,
        action: &'static str,
        delivery_status: ReminderDeliveryStatus,
        reason_code: ReasonCodeId,
        escalation_level: Option<u8>,
    ) -> Result<Ph1RemResponse, StorageError> {
        let mut state = self.lock_state()?;

        let (row_state, occ_scheduled_time) = {
            let row = state.reminders.get(reminder_id).ok_or_else(|| {
                StorageError::ContractViolation(ContractViolation::InvalidValue {
                    field: "ph1rem.delivery.reminder_id",
                    reason: "not found",
                })
            })?;

            if row.tenant_id != *tenant_id || row.user_id != *user_id {
                return Ok(Ph1RemResponse::Refuse(Ph1RemRefuse::v1(
                    req.envelope.simulation_id.clone(),
                    reason_codes::REM_FAIL_SCOPE_VIOLATION,
                    "tenant_id/user_id do not match reminder owner".to_string(),
                )?));
            }

            let occ = state.occurrences.get(occurrence_id).ok_or_else(|| {
                StorageError::ContractViolation(ContractViolation::InvalidValue {
                    field: "ph1rem.delivery.occurrence_id",
                    reason: "not found",
                })
            })?;

            if occ.reminder_id != *reminder_id {
                return Ok(Ph1RemResponse::Refuse(Ph1RemRefuse::v1(
                    req.envelope.simulation_id.clone(),
                    reason_codes::REM_FAIL_SCOPE_VIOLATION,
                    "occurrence does not belong to reminder_id".to_string(),
                )?));
            }
            (row.state, occ.scheduled_time)
        };

        let idx = (
            tenant_id.clone(),
            reminder_id.clone(),
            occurrence_id.clone(),
            delivery_attempt_id.clone(),
            action,
        );

        if let Some(existing_idx) = state.delivery_idempotency.get(&idx).cloned() {
            let existing = state.delivery_attempts.get(existing_idx).ok_or(
                StorageError::ContractViolation(ContractViolation::InvalidValue {
                    field: "ph1rem.delivery.idempotency",
                    reason: "existing delivery row missing",
                }),
            )?;
            return Ok(Ph1RemResponse::Ok(Ph1RemOk::v1(
                req.envelope.simulation_id.clone(),
                reason_code,
                reminder_id.clone(),
                Some(occurrence_id.clone()),
                row_state,
                Some(occ_scheduled_time),
                Some(existing.status),
                Some(existing.attempt_id.clone()),
                existing.proof_ref.clone(),
                escalation_level,
            )?));
        }

        let proof_ref = Some(format!(
            "proof:{}:{}",
            delivery_attempt_id.as_str(),
            req.envelope.now.0
        ));

        let entry = ReminderDeliveryAttemptRow {
            attempt_id: delivery_attempt_id.clone(),
            reminder_id: reminder_id.clone(),
            occurrence_id: occurrence_id.clone(),
            channel,
            status: delivery_status,
            proof_ref: proof_ref.clone(),
            created_at: req.envelope.now,
        };

        state.delivery_attempts.push(entry);
        let created_idx = state.delivery_attempts.len() - 1;
        state.delivery_idempotency.insert(idx, created_idx);
        state.update_idempotency.insert(
            (
                tenant_id.clone(),
                reminder_id.clone(),
                idempotency_key.to_string(),
                action,
            ),
            reminder_id.clone(),
        );

        Ok(Ph1RemResponse::Ok(Ph1RemOk::v1(
            req.envelope.simulation_id.clone(),
            reason_code,
            reminder_id.clone(),
            Some(occurrence_id.clone()),
            row_state,
            Some(occ_scheduled_time),
            Some(delivery_status),
            Some(delivery_attempt_id.clone()),
            proof_ref,
            escalation_level,
        )?))
    }

    fn lock_state(&self) -> Result<std::sync::MutexGuard<'_, Ph1RemState>, StorageError> {
        self.state.lock().map_err(|_| {
            StorageError::ContractViolation(ContractViolation::InvalidValue {
                field: "ph1rem.state",
                reason: "runtime state lock poisoned",
            })
        })
    }
}

fn resolve_desired_time(now: MonotonicTimeNs, value: &str) -> Option<MonotonicTimeNs> {
    let s = value.trim().to_ascii_lowercase();
    if s.is_empty() {
        return None;
    }

    if s == "now" || s == "today" {
        return Some(now);
    }
    if s.contains("tomorrow") {
        return Some(MonotonicTimeNs(now.0.saturating_add(86_400_000_000_000)));
    }

    if let Some(delta_ns) = parse_relative_delay_ns(&s) {
        return Some(MonotonicTimeNs(now.0.saturating_add(delta_ns)));
    }

    if let Ok(raw) = s.parse::<u64>() {
        // len>=16 => ns, len>=13 => ms, len>=10 => sec
        let ns = if s.len() >= 16 {
            raw
        } else if s.len() >= 13 {
            raw.saturating_mul(1_000_000)
        } else if s.len() >= 10 {
            raw.saturating_mul(1_000_000_000)
        } else {
            return None;
        };
        if ns > 0 {
            return Some(MonotonicTimeNs(ns));
        }
    }

    None
}

fn parse_relative_delay_ns(s: &str) -> Option<u64> {
    let stripped = s.strip_prefix("in ")?;
    let mut parts = stripped.split_whitespace();
    let amount = parts.next()?.parse::<u64>().ok()?;
    let unit = parts.next()?;
    if amount == 0 {
        return None;
    }

    let factor = match unit {
        "s" | "sec" | "secs" | "second" | "seconds" => 1_000_000_000,
        "m" | "min" | "mins" | "minute" | "minutes" => 60_000_000_000,
        "h" | "hr" | "hrs" | "hour" | "hours" => 3_600_000_000_000,
        "d" | "day" | "days" => 86_400_000_000_000,
        _ => return None,
    };

    Some(amount.saturating_mul(factor))
}

#[cfg(test)]
mod tests {
    use super::*;
    use selene_kernel_contracts::ph1_voice_id::UserId;
    use selene_kernel_contracts::ph1j::DeviceId;
    use selene_kernel_contracts::ph1position::TenantId;
    use selene_kernel_contracts::ph1rem::{
        Ph1RemRequest, ReminderChannel, ReminderLocalTimeMode, ReminderPriorityLevel, ReminderType,
    };
    use selene_storage::ph1f::{DeviceRecord, IdentityRecord, IdentityStatus};

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

    #[test]
    fn at_rem_01_schedule_idempotent() {
        let mut store = seeded_store();
        let runtime = Ph1RemRuntime::default();
        let req = Ph1RemRequest::schedule_commit_v1(
            selene_kernel_contracts::ph1j::CorrelationId(101),
            selene_kernel_contracts::ph1j::TurnId(201),
            MonotonicTimeNs(1_000_000_000),
            TenantId::new("tenant_demo").unwrap(),
            UserId::new("tenant_demo:user_1").unwrap(),
            Some(DeviceId::new("device_1").unwrap()),
            ReminderType::Task,
            "call payroll".to_string(),
            "in 15 minutes".to_string(),
            "America/Los_Angeles".to_string(),
            ReminderLocalTimeMode::LocalTime,
            ReminderPriorityLevel::Normal,
            None,
            vec![ReminderChannel::Text],
            "idem_1".to_string(),
        )
        .unwrap();

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
        let runtime = Ph1RemRuntime::default();
        let req = Ph1RemRequest::schedule_commit_v1(
            selene_kernel_contracts::ph1j::CorrelationId(102),
            selene_kernel_contracts::ph1j::TurnId(202),
            MonotonicTimeNs(1_000_000_000),
            TenantId::new("tenant_demo").unwrap(),
            UserId::new("tenant_demo:user_1").unwrap(),
            Some(DeviceId::new("device_1").unwrap()),
            ReminderType::Task,
            "call payroll".to_string(),
            "later sometime maybe".to_string(),
            "America/Los_Angeles".to_string(),
            ReminderLocalTimeMode::LocalTime,
            ReminderPriorityLevel::Normal,
            None,
            vec![ReminderChannel::Text],
            "idem_2".to_string(),
        )
        .unwrap();

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
}
