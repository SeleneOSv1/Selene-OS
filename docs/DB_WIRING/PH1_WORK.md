# PH1_WORK DB Wiring (Design vNext)

## A) Engine Header
- engine_id: PH1.WORK
- layer: Enterprise Support
- authority: Authoritative (work-order ledger decision only)
- role: Deterministic append-only WorkOrder ledger boundary (`WORK_POLICY_EVALUATE -> WORK_DECISION_COMPUTE`)
- placement: ENTERPRISE_SUPPORT (OS-internal persistence/control path)

## B) Ownership
- Tables owned: NONE in this runtime slice (decision-only runtime).
- Canonical persistence remains in `SELENE_OS_CORE_TABLES` (`work_order_ledger` truth + `work_orders_current` materialization).
- Reads:
  - bounded work-order event input (`tenant_id`, `work_order_id`, `event_type`, `payload_min`, `created_at`, optional `idempotency_key`)
  - deterministic violation/dedupe flags from Selene OS + storage boundary checks.
- Writes:
  - no direct table writes in this runtime slice.
  - emits deterministic decision output only (`OK | REFUSED | FAIL`) with reason code and event-id/no-op posture.

## C) Hard Boundaries
- Append-only rule is mandatory: corrections are new events; no in-place mutation.
- Idempotency duplicates must resolve to deterministic no-op with the previously assigned event id.
- Canonical replay ordering is by `work_order_event_id`.
- PH1.WORK must never silently merge conflicting updates; conflicts are clarified at OS level and recorded.
- PH1.WORK must never execute side effects, permissions, or workflow steps.

## D) Wiring
- Invoked_by: Selene OS around each WorkOrder ledger append decision.
- Inputs_from:
  - WorkOrder event intent (`event_type`, `payload_min`, `created_at`, idempotency posture)
  - persisted-state guard outcomes (`append_only_violation`, `tenant_scope_mismatch`, `idempotency_duplicate`)
  - deterministic replay invariants (`deterministic_replay_order=true`, `no_silent_conflict_merge=true`)
- Outputs_to:
  - `work_policy_bundle` (`event_valid`, `append_allowed`, duplicate/violation flags, `reason_code`)
  - `work_decision_bundle` (`status`, optional `work_order_event_id`, `idempotency_no_op`, `reason_code`)
- Invocation_condition: ENTERPRISE_SUPPORT (work-order governance enabled)
- Deterministic sequence:
  - `WORK_POLICY_EVALUATE`:
    - validates bounded event input and idempotency requirements.
    - computes deterministic append posture (`append_allowed` or violation/duplicate posture).
  - `WORK_DECISION_COMPUTE`:
    - emits exactly one decision (`OK | REFUSED | FAIL`).
    - duplicate path emits `OK` + `idempotency_no_op=true` with previous `work_order_event_id`.
    - append path emits `OK` + new `work_order_event_id`.
- Not allowed:
  - engine-to-engine direct calls
  - non-deterministic event-id assignment behavior
  - bypassing append-only/tenant-scope checks

## E) Related Engine Boundaries
- `SELENE_OS_CORE_TABLES`: canonical storage truth for `work_order_ledger` and `work_orders_current` remains external and authoritative.
- `PH1.SCHED`: scheduler consumes persisted step/attempt state; PH1.WORK preserves replay-safe event ordering and idempotency semantics.
- `PH1.LEASE`: lease decisions gate executor ownership; PH1.WORK records resulting state changes deterministically.
- `PH1.EXPORT`: compliance export consumes work-order ledger truth generated through this boundary.
- `PH1.J`: all work-order decisions are reason-coded and auditable.

## F) Acceptance Tests
- AT-WORK-01: append-only enforcement.
- AT-WORK-02: current-view rebuild matches ledger replay order.
- AT-WORK-03: idempotency duplicate resolves to deterministic no-op.
- AT-WORK-04: tenant mismatch is blocked.
