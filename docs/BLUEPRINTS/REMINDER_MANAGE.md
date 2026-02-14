# REMINDER_MANAGE Blueprint Record

## 1) Blueprint Header
- `process_id`: `REMINDER_MANAGE`
- `intent_type`: `REMINDER_MANAGE`
- `version`: `v1`
- `status`: `ACTIVE`

## 1A) Contract Boundary
- This blueprint defines orchestration flow only.
- PH1.REM owns deterministic reminder timing mechanics only; PH1.BCAST owns message lifecycle/content.
- For `reminder_type=BCAST_MHP_FOLLOWUP`, Selene OS must resume PH1.BCAST at `REMINDER_SET -> REMINDER_FIRED`.

## 2) Required Inputs
- `tenant_id`
- `requester_user_id`
- `reminder_action` (`SCHEDULE | UPDATE | CANCEL | SNOOZE | DELIVER_PRE | DELIVER_DUE | SCHEDULE_FOLLOWUP | SCHEDULE_RETRY | ESCALATE | MARK_COMPLETED | MARK_FAILED`)
- `reminder_type` (`TASK | MEETING | TIMER | MEDICAL | CUSTOM | BCAST_MHP_FOLLOWUP`)
- `reminder_id` (required for non-`SCHEDULE` actions)
- `occurrence_id` (required for occurrence-scoped actions)
- `desired_time` (required for `SCHEDULE`)
- `user_timezone` + `local_time_mode` (required for `SCHEDULE`)
- `priority_level` (required for `SCHEDULE`)
- `broadcast_id`, `recipient_id`, `prompt_dedupe_key` (required when `reminder_type=BCAST_MHP_FOLLOWUP`)
- `idempotency_key`

## 3) Success Output Schema
```text
reminder_id: string
occurrence_id: string (optional)
state: enum (SCHEDULED | FOLLOWUP_PENDING | SNOOZED | COMPLETED | CANCELED | ESCALATED | FAILED)
delivery_status: enum (DELIVERED | DEFERRED_QUIET_HOURS | RETRY_SCHEDULED | FAIL) (optional)
reason_code: string
```

## 4) Ordered Engine Steps
| step_id | engine_name | capability_id | required_fields | produced_fields | side_effects | timeout_ms | max_retries | retry_backoff_ms | retryable_reason_codes |
|---|---|---|---|---|---|---:|---:|---:|---|
| REM_S01 | PH1.X | PH1X_CONFIRM_COMMIT_ROW | reminder_action, reminder_type, reminder_id?, occurrence_id? | confirmation_state | DB_WRITE | 300 | 1 | 100 | [OS_CONFIRM_TIMEOUT] |
| REM_S02 | PH1.REM | PH1REM_SCHEDULE_COMMIT_ROW | reminder_action=SCHEDULE, tenant_id, requester_user_id, reminder_type, desired_time, user_timezone, local_time_mode, priority_level, idempotency_key | reminder_id, occurrence_id, scheduled_time, state | DB_WRITE (simulation-gated) | 700 | 2 | 250 | [REM_FAIL_TIME_AMBIGUOUS_NEEDS_CONFIRM, REM_FAIL_TIMEZONE_INVALID] |
| REM_S03 | PH1.REM | PH1REM_UPDATE_COMMIT_ROW | reminder_action=UPDATE, tenant_id, requester_user_id, reminder_id, updated_fields, idempotency_key | reminder_id, updated_fields, state | DB_WRITE (simulation-gated) | 700 | 2 | 250 | [REM_FAIL_STATE_TRANSITION_INVALID] |
| REM_S04 | PH1.REM | PH1REM_CANCEL_COMMIT_ROW | reminder_action=CANCEL, tenant_id, requester_user_id, reminder_id, idempotency_key | reminder_id, state=CANCELED | DB_WRITE (simulation-gated) | 700 | 2 | 250 | [REM_FAIL_STATE_TRANSITION_INVALID] |
| REM_S05 | PH1.REM | PH1REM_SNOOZE_COMMIT_ROW | reminder_action=SNOOZE, tenant_id, requester_user_id, reminder_id, occurrence_id, snooze_duration_ms, idempotency_key | reminder_id, occurrence_id, snooze_until_ms, state=SNOOZED | DB_WRITE (simulation-gated) | 700 | 2 | 250 | [REM_FAIL_STATE_TRANSITION_INVALID] |
| REM_S06 | PH1.REM | PH1REM_FOLLOWUP_SCHEDULE_COMMIT_ROW | reminder_action=SCHEDULE_FOLLOWUP, tenant_id, requester_user_id, reminder_id, occurrence_id, followup_delay_ms, idempotency_key | reminder_id, occurrence_id, followup_time_ms, state=FOLLOWUP_PENDING | DB_WRITE (simulation-gated) | 700 | 2 | 250 | [REM_FAIL_STATE_TRANSITION_INVALID] |
| REM_S07 | PH1.REM | PH1REM_DELIVERY_RETRY_SCHEDULE_COMMIT_ROW | reminder_action=SCHEDULE_RETRY, tenant_id, requester_user_id, reminder_id, occurrence_id, retry_time_ms, idempotency_key | reminder_id, occurrence_id, retry_time_ms | DB_WRITE (simulation-gated) | 700 | 2 | 250 | [REM_FAIL_STATE_TRANSITION_INVALID] |
| REM_S08 | PH1.REM | PH1REM_DELIVER_PRE_COMMIT_ROW | reminder_action=DELIVER_PRE, tenant_id, requester_user_id, reminder_id, occurrence_id, delivery_channel, delivery_attempt_id, idempotency_key | delivery_status, delivery_proof_ref | DB_WRITE + EXTERNAL_SEND_REQUEST (simulation-gated) | 700 | 2 | 250 | [REM_FAIL_QUIET_HOURS_DEFERRED] |
| REM_S09 | PH1.REM | PH1REM_DELIVER_DUE_COMMIT_ROW | reminder_action=DELIVER_DUE, tenant_id, requester_user_id, reminder_id, occurrence_id, delivery_channel, delivery_attempt_id, offline_state, idempotency_key | delivery_status, delivery_proof_ref | DB_WRITE + EXTERNAL_SEND_REQUEST (simulation-gated) | 700 | 2 | 250 | [REM_FAIL_QUIET_HOURS_DEFERRED] |
| REM_S10 | PH1.REM | PH1REM_ESCALATE_COMMIT_ROW | reminder_action=ESCALATE, tenant_id, requester_user_id, reminder_id, occurrence_id, from_channel, to_channel, delivery_attempt_id, idempotency_key | delivery_status, escalation_level, delivery_proof_ref | DB_WRITE + EXTERNAL_SEND_REQUEST (simulation-gated) | 700 | 2 | 250 | [REM_FAIL_STATE_TRANSITION_INVALID] |
| REM_S11 | PH1.REM | PH1REM_MARK_COMPLETED_COMMIT_ROW | reminder_action=MARK_COMPLETED, tenant_id, requester_user_id, reminder_id, occurrence_id, ack_source, idempotency_key | state=COMPLETED, completion_time_ms | DB_WRITE (simulation-gated) | 700 | 2 | 250 | [REM_FAIL_STATE_TRANSITION_INVALID] |
| REM_S12 | PH1.REM | PH1REM_MARK_FAILED_COMMIT_ROW | reminder_action=MARK_FAILED, tenant_id, requester_user_id, reminder_id, occurrence_id, failure_reason, idempotency_key | state=FAILED | DB_WRITE (simulation-gated) | 700 | 2 | 250 | [REM_FAIL_STATE_TRANSITION_INVALID] |
| REM_S13 | PH1.BCAST | BCAST_DELIVER_COMMIT | reminder_type=BCAST_MHP_FOLLOWUP, reminder_action=DELIVER_DUE, broadcast_id, recipient_id, delivery_plan_ref, simulation_context, idempotency_key | delivery_request_ref, recipient_state | EXTERNAL_DELIVERY_REQUEST (simulation-gated) | 700 | 2 | 250 | [BCAST_SIMULATION_CONTEXT_MISSING, BCAST_DELIVERY_PLAN_INVALID] |

## 5) Confirmation Points
- `REM_S01` confirms user intent before schedule/update/cancel/escalate paths.
- For `reminder_type=BCAST_MHP_FOLLOWUP`, confirmation must include `due_at` and target `recipient_id`.

## 6) Simulation Requirements
- `REMINDER_SCHEDULE_COMMIT`
- `REMINDER_UPDATE_COMMIT`
- `REMINDER_CANCEL_COMMIT`
- `REMINDER_SNOOZE_COMMIT`
- `REMINDER_DELIVER_PRE_COMMIT`
- `REMINDER_DELIVER_DUE_COMMIT`
- `REMINDER_FOLLOWUP_SCHEDULE_COMMIT`
- `REMINDER_ESCALATE_COMMIT`
- `REMINDER_DELIVERY_RETRY_SCHEDULE_COMMIT`
- `REMINDER_MARK_COMPLETED_COMMIT`
- `REMINDER_MARK_FAILED_COMMIT`
- `BCAST_DELIVER_COMMIT`

## 7) Refusal Conditions
- identity unknown/unverified -> `REM_FAIL_SCOPE_VIOLATION`
- invalid action/state transition -> `REM_FAIL_STATE_TRANSITION_INVALID`
- missing required fields for selected action -> `NLP_CLARIFY_MISSING_FIELD`
- missing `broadcast_id|recipient_id|prompt_dedupe_key` when `reminder_type=BCAST_MHP_FOLLOWUP` -> `REM_FAIL_SCOPE_VIOLATION`

## 8) Acceptance Tests
- `AT-PBS-REM-01`: All side-effect steps map to declared simulations (No Simulation -> No Execution).
- `AT-PBS-REM-02`: `BCAST_MHP_FOLLOWUP` scheduling requires `(broadcast_id, recipient_id, due_at, prompt_dedupe_key)` and uses deterministic idempotency.
- `AT-PBS-REM-03`: Reminder fire with `BCAST_MHP_FOLLOWUP` resumes PH1.BCAST via `BCAST_DELIVER_COMMIT` only.
- `AT-PBS-REM-04`: Invalid state transitions fail closed with deterministic reason codes.
