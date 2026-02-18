# PH1_SCHED DB Wiring (Design vNext)

## A) Engine Header
- engine_id: PH1.SCHED
- layer: Enterprise Support
- authority: Authoritative (deterministic scheduler decisions)
- role: Deterministic retry/wait/fail decision engine (`SCHED_POLICY_EVALUATE -> SCHED_DECISION_COMPUTE`)
- placement: ENTERPRISE_SUPPORT (OS-internal, policy/contract-gated)

## B) Ownership
- Tables owned: NONE in current runtime slice (decision-only runtime)
- Reads:
  - bounded scheduler request context (`tenant_id`, `work_order_id`, `step_id`, `attempt_index`)
  - step policy parameters (`timeout_ms`, `max_retries`, `retry_backoff_ms`, `retryable_reason_codes`)
  - optional `last_failure_reason_code`
- Writes:
  - no direct table writes in this runtime slice
  - emits deterministic `RETRY_AT | FAIL | WAIT` decisions only

## C) Hard Boundaries
- Must never generate random jitter.
- Must never retry past `max_retries`.
- `WAIT` must never advance attempt index or plan state.
- Must fail closed when schema/bounds are invalid.
- Must never execute side effects or bypass Access/Simulation ordering.

## D) Wiring
- Invoked_by: Selene OS orchestration path (post-step failure/timeout handling).
- Inputs_from:
  - WorkOrder context (`tenant_id`, `work_order_id`, `step_id`)
  - scheduler bounds (`timeout_ms`, `max_retries`, `retry_backoff_ms`)
  - attempt/failure context (`attempt_index`, `last_failure_reason_code`, `retryable_reason_codes`)
- Outputs_to:
  - scheduler policy evaluation bundle (retry eligibility + guard flags)
  - scheduler decision bundle (`action`, `next_due_at`, `attempt_next_index`, `reason_code`)
- Invocation_condition: ENTERPRISE_SUPPORT (scheduler enabled for WorkOrder execution flow)
- Deterministic sequence:
  - `SCHED_POLICY_EVALUATE`:
    - evaluates timeout guard, retry budget guard, and retryable-reason eligibility
    - emits deterministic `retry_allowed` and `next_attempt_index`
  - `SCHED_DECISION_COMPUTE`:
    - outputs exactly one action: `RETRY_AT`, `FAIL`, or `WAIT`
    - when `RETRY_AT`, emits deterministic `next_due_at`
    - when `WAIT`, enforces `attempt_next_index == attempt_index`
- Not allowed:
  - engine-to-engine direct calls
  - random or heuristic jitter
  - hidden retries outside declared policy bounds

## E) Related Engine Boundaries
- SELENE_OS_CORE_TABLES / PH1.WORK:
  - PH1.SCHED outputs are consumed by Selene OS to drive append-only WorkOrder ledger progression.
  - Scheduler output is advisory to orchestration; state transition commits remain in WorkOrder ledger paths.
- PH1.LEASE:
  - retry/takeover sequencing uses lease ownership outside PH1.SCHED; PH1.SCHED never mints/renews leases.
- PH1.EXPORT:
  - scheduler decisions are exportable via compliance traces but PH1.SCHED itself does not persist export artifacts.

## F) Acceptance Tests
- AT-SCHED-01: Retry schedule is deterministic.
- AT-SCHED-02: Max retries are enforced.
- AT-SCHED-03: WAIT does not advance plan and remains auditable.
