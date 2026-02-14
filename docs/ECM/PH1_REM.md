PH1.REM ECM Spec
Engine Header
engine_id: PH1.REM
purpose: Deterministic reminder scheduling/firing timing mechanics with bounded recurrence, follow-ups, retries, and delivery attempt proof recording. Timing-only for BCAST.MHP follow-ups; message lifecycle/content remains in PH1.BCAST.
data_owned: reminder tables (see docs/DB_WIRING/PH1_REM.md; KC.17)
version: v1
status: ACTIVE
allowed_callers: SELENE_OS_ONLY

Capability List
### `PH1REM_SCHEDULE_COMMIT_ROW`
input_schema: (now, tenant_id, user_id, device_id, reminder_type, desired_time, user_timezone, local_time_mode, priority_level, recurrence_rule?, channel_preferences, idempotency_key)
output_schema: Result<(reminder_id, occurrence_id, scheduled_time, state), StorageError>
side_effects: DECLARED (DB_WRITE)
simulation_gated: REMINDER_SCHEDULE_COMMIT

### `PH1REM_UPDATE_COMMIT_ROW`
input_schema: (now, tenant_id, user_id, reminder_id, updated_fields, idempotency_key)
output_schema: Result<(reminder_id, updated_fields, state), StorageError>
side_effects: DECLARED (DB_WRITE)
simulation_gated: REMINDER_UPDATE_COMMIT

### `PH1REM_CANCEL_COMMIT_ROW`
input_schema: (now, tenant_id, user_id, reminder_id, cancel_reason?, idempotency_key)
output_schema: Result<(reminder_id, state=CANCELED), StorageError>
side_effects: DECLARED (DB_WRITE)
simulation_gated: REMINDER_CANCEL_COMMIT

### `PH1REM_SNOOZE_COMMIT_ROW`
input_schema: (now, tenant_id, user_id, reminder_id, occurrence_id, snooze_duration_ms, idempotency_key)
output_schema: Result<(reminder_id, occurrence_id, snooze_until_ms, state=SNOOZED), StorageError>
side_effects: DECLARED (DB_WRITE)
simulation_gated: REMINDER_SNOOZE_COMMIT

### `PH1REM_FOLLOWUP_SCHEDULE_COMMIT_ROW`
input_schema: (now, tenant_id, user_id, reminder_id, occurrence_id, followup_delay_ms, idempotency_key)
output_schema: Result<(reminder_id, occurrence_id, followup_time_ms, state=FOLLOWUP_PENDING), StorageError>
side_effects: DECLARED (DB_WRITE)
simulation_gated: REMINDER_FOLLOWUP_SCHEDULE_COMMIT

### `PH1REM_DELIVERY_RETRY_SCHEDULE_COMMIT_ROW`
input_schema: (now, tenant_id, user_id, reminder_id, occurrence_id, retry_time_ms, idempotency_key)
output_schema: Result<(reminder_id, occurrence_id, retry_time_ms), StorageError>
side_effects: DECLARED (DB_WRITE)
simulation_gated: REMINDER_DELIVERY_RETRY_SCHEDULE_COMMIT

### `PH1REM_DELIVER_PRE_COMMIT_ROW`
input_schema: (now, tenant_id, user_id, reminder_id, occurrence_id, delivery_channel, delivery_attempt_id, idempotency_key)
output_schema: Result<(delivery_status, delivery_proof_ref?), StorageError>
side_effects: DECLARED (DB_WRITE + EXTERNAL_SEND_REQUEST)
simulation_gated: REMINDER_DELIVER_PRE_COMMIT

### `PH1REM_DELIVER_DUE_COMMIT_ROW`
input_schema: (now, tenant_id, user_id, reminder_id, occurrence_id, delivery_channel, delivery_attempt_id, offline_state, idempotency_key)
output_schema: Result<(delivery_status, delivery_proof_ref?), StorageError>
side_effects: DECLARED (DB_WRITE + EXTERNAL_SEND_REQUEST)
simulation_gated: REMINDER_DELIVER_DUE_COMMIT

### `PH1REM_ESCALATE_COMMIT_ROW`
input_schema: (now, tenant_id, user_id, reminder_id, occurrence_id, from_channel, to_channel, delivery_attempt_id, idempotency_key)
output_schema: Result<(delivery_status, escalation_level, delivery_proof_ref?), StorageError>
side_effects: DECLARED (DB_WRITE + EXTERNAL_SEND_REQUEST)
simulation_gated: REMINDER_ESCALATE_COMMIT

### `PH1REM_MARK_COMPLETED_COMMIT_ROW`
input_schema: (now, tenant_id, user_id, reminder_id, occurrence_id, ack_source, idempotency_key)
output_schema: Result<(state=COMPLETED, completion_time_ms), StorageError>
side_effects: DECLARED (DB_WRITE)
simulation_gated: REMINDER_MARK_COMPLETED_COMMIT

### `PH1REM_MARK_FAILED_COMMIT_ROW`
input_schema: (now, tenant_id, user_id, reminder_id, occurrence_id, failure_reason, idempotency_key)
output_schema: Result<(state=FAILED), StorageError>
side_effects: DECLARED (DB_WRITE)
simulation_gated: REMINDER_MARK_FAILED_COMMIT

Failure Modes + Reason Codes
deterministic failures include:
REM_FAIL_TIME_AMBIGUOUS_NEEDS_CONFIRM
REM_FAIL_TIMEZONE_INVALID
REM_FAIL_RECURRENCE_INVALID
REM_FAIL_QUIET_HOURS_DEFERRED
REM_FAIL_IDEMPOTENCY_REPLAY
REM_FAIL_SCOPE_VIOLATION
REM_FAIL_STATE_TRANSITION_INVALID

Hard Rules
PH1.REM must not decide message urgency/classification/content.
For BCAST_MHP_FOLLOWUP, PH1.REM stores only timing/linkage fields and Selene OS resumes PH1.BCAST at REMINDER_SET -> REMINDER_FIRED.

Sources
docs/DB_WIRING/PH1_REM.md
docs/DB_WIRING/PH1_BCAST.md (Section BCAST.MHP.REM)
