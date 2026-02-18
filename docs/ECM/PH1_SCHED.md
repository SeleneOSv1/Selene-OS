# PH1_SCHED ECM (Design vNext)

## Engine Header
- engine_id: PH1.SCHED
- role: deterministic scheduler decision engine for WorkOrder retry/wait/fail control
- placement: ENTERPRISE_SUPPORT

## Capability List

### capability_id: SCHED_POLICY_EVALUATE
- input_schema:
  - bounded envelope (`correlation_id`, `turn_id`, `max_retryable_reason_codes`, `max_diagnostics`, `max_backoff_ms`)
  - WorkOrder context (`tenant_id`, `work_order_id`, `step_id`)
  - timing and policy (`now_ns`, `step_started_at_ns`, `timeout_ms`, `max_retries`, `retry_backoff_ms`)
  - failure context (`attempt_index`, optional `last_failure_reason_code`, `retryable_reason_codes`)
  - hard rule flag (`wait_is_pause_only=true`)
- output_schema:
  - deterministic policy status (`timeout_exceeded`, `max_retries_reached`, `retry_allowed`)
  - normalized attempt indexes (`attempt_index`, `next_attempt_index`)
  - bounded retry parameters (`retry_backoff_ms`)
  - reason code for scheduler posture
- allowed_callers: SELENE_OS_ONLY
- side_effects: NONE
- failure_modes: INPUT_SCHEMA_INVALID, UPSTREAM_INPUT_MISSING, SCHED_TIMEOUT, SCHED_MAX_RETRIES_REACHED, SCHED_NOT_RETRYABLE, BUDGET_EXCEEDED, INTERNAL_PIPELINE_ERROR
- reason_codes:
  - PH1_SCHED_OK_POLICY_EVALUATE
  - SCHED_TIMEOUT
  - SCHED_MAX_RETRIES_REACHED
  - SCHED_NOT_RETRYABLE
  - PH1_SCHED_INPUT_SCHEMA_INVALID
  - PH1_SCHED_UPSTREAM_INPUT_MISSING
  - PH1_SCHED_BUDGET_EXCEEDED
  - PH1_SCHED_INTERNAL_PIPELINE_ERROR

### capability_id: SCHED_DECISION_COMPUTE
- input_schema:
  - bounded envelope (`correlation_id`, `turn_id`, `max_retryable_reason_codes`, `max_diagnostics`, `max_backoff_ms`)
  - WorkOrder context (`tenant_id`, `work_order_id`, `step_id`)
  - normalized policy fields from evaluate step (`timeout_exceeded`, `max_retries_reached`, `retry_allowed`)
  - attempt/timing fields (`attempt_index`, `next_attempt_index`, `retry_backoff_ms`, `now_ns`)
  - hard rule flag (`wait_is_pause_only=true`)
- output_schema:
  - action (`RETRY_AT | FAIL | WAIT`)
  - `next_due_at_ns` only for `RETRY_AT`
  - deterministic attempt progression (`attempt_next_index`)
  - explicit invariants (`wait_is_pause_only=true`, `deterministic=true`)
- allowed_callers: SELENE_OS_ONLY
- side_effects: NONE
- failure_modes: INPUT_SCHEMA_INVALID, SCHED_TIMEOUT, SCHED_MAX_RETRIES_REACHED, SCHED_NOT_RETRYABLE, BUDGET_EXCEEDED, INTERNAL_PIPELINE_ERROR
- reason_codes:
  - PH1_SCHED_OK_DECISION_COMPUTE
  - SCHED_RETRY_SCHEDULED
  - SCHED_TIMEOUT
  - SCHED_MAX_RETRIES_REACHED
  - SCHED_NOT_RETRYABLE
  - PH1_SCHED_INPUT_SCHEMA_INVALID
  - PH1_SCHED_BUDGET_EXCEEDED
  - PH1_SCHED_INTERNAL_PIPELINE_ERROR

## Constraints
- Engines never call engines directly; Selene OS orchestrates all sequencing.
- Scheduler decisions are deterministic for identical inputs (no random jitter).
- `WAIT` action must not advance attempt index.
- Scheduler output does not execute work; it only shapes OS orchestration posture.

## Related Engine Boundaries
- SELENE_OS_CORE_TABLES / PH1.WORK boundary:
  - scheduler decisions are consumed by WorkOrder orchestration and ledger append logic outside PH1.SCHED runtime.
- PH1.LEASE boundary:
  - lease acquisition/renew/release remains outside PH1.SCHED.
- PH1.EXPORT boundary:
  - scheduler decisions may be included in export traces, but PH1.SCHED does not own export artifact generation.
