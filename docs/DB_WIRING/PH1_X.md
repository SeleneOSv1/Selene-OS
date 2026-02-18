# PH1.X DB Wiring Spec

## 1) Engine Header

- `engine_id`: `PH1.X`
- `purpose`: Persist deterministic PH1.X conversational directives (`confirm`, `clarify`, `respond`, `dispatch`, `wait`) as bounded audit events without introducing PH1.X-owned tables.
- `version`: `v1`
- `status`: `PASS`

## 2) Data Owned (authoritative)

### `os_core.audit_events`
- `truth_type`: `LEDGER`
- `primary key`: `event_id`
- invariants:
  - PH1.X outcomes are recorded with `engine=PH1.X`
  - `confirm` uses `event_type=XConfirm`
  - `dispatch` uses `event_type=XDispatch`
  - `clarify/respond/wait` use `event_type=Other` with bounded directive payload keys
  - payload values are bounded and reason-coded
  - idempotent append dedupe on `(correlation_id, idempotency_key)`
  - append-only; overwrite/delete prohibited

## 3) Reads (dependencies)

### Identity/device/session scope checks
- reads: `identities`, `devices`, `sessions`
- keys/joins used: direct FK existence + deterministic scope check `(session.user_id, session.device_id)`
- required indices:
  - `identities(user_id)` (PK)
  - `devices(device_id)` (PK)
  - `sessions(session_id)` (PK)
- scope rules:
  - device must belong to `user_id`
  - one tenant binding per `device_id` for PH1.X rows
- why this read is required: fail closed before PH1.X audit writes

### Replay reads
- reads: `audit_events` by `correlation_id` and `tenant_id`
- keys/joins used: deterministic correlation chain reads
- required indices:
  - `audit_events(correlation_id, turn_id)`
  - `audit_events(tenant_id, created_at)` (or equivalent tenant filter path)
- scope rules: no cross-tenant writes; tenant attribution required
- why this read is required: deterministic replay and dedupe verification

### WorkOrder + lease scope checks (dispatch gating references)
- reads:
  - `work_orders_current` by `(tenant_id, work_order_id)` for current status
  - `work_order_leases` by `(tenant_id, work_order_id)` for active lease ownership
- keys/joins used:
  - deterministic key lookup on `work_orders_current(tenant_id, work_order_id)`
  - active lease filter `lease_state='ACTIVE'` with latest `lease_expires_at`
- required indices:
  - `ux_work_orders_current_tenant_work_order`
  - `ux_work_order_leases_tenant_work_order_idempotency` (tenant/work-order key path)
- scope rules:
  - PH1.X reads only in-tenant work-order rows bound to the current `correlation_id`
  - PH1.X does not mutate work-order/lease tables; writes remain owned by `SELENE_OS_CORE_TABLES`
- why this read is required:
  - deterministic clarify/confirm/dispatch gating against current WorkOrder status
  - deterministic no-dispatch rule when lease is missing/expired

## 4) Writes (outputs)

### Commit `confirm`
- writes: `audit_events`
- required fields:
  - `tenant_id`, `engine=PH1.X`, `event_type=XConfirm`, `reason_code`, `correlation_id`, `turn_id`
  - payload: `directive=confirm`, `confirm_kind`, `work_order_id`, `work_order_status_snapshot`, `pending_state`
- idempotency_key rule (exact formula):
  - dedupe key = `(correlation_id, idempotency_key)`

### Commit `clarify`
- writes: `audit_events`
- required fields:
  - `tenant_id`, `engine=PH1.X`, `event_type=Other`, `reason_code`, `correlation_id`, `turn_id`
  - payload: `directive=clarify`, `what_is_missing`, `clarification_unit_id`, `work_order_id`, `work_order_status_snapshot`, `pending_state`
- idempotency_key rule (exact formula):
  - dedupe key = `(correlation_id, idempotency_key)`

### Commit `respond`
- writes: `audit_events`
- required fields:
  - `tenant_id`, `engine=PH1.X`, `event_type=Other`, `reason_code`, `correlation_id`, `turn_id`
  - payload: `directive=respond`, `response_kind`, `work_order_id`, `work_order_status_snapshot`, `pending_state`
- idempotency_key rule (exact formula):
  - dedupe key = `(correlation_id, idempotency_key)`

### Commit `dispatch`
- writes: `audit_events`
- required fields:
  - `tenant_id`, `engine=PH1.X`, `event_type=XDispatch`, `reason_code`, `correlation_id`, `turn_id`
  - payload: `directive=dispatch`, `dispatch_target`, `work_order_id`, `work_order_status_snapshot`, `pending_state`, `lease_token_hash?`
- idempotency_key rule (exact formula):
  - dedupe key = `(correlation_id, idempotency_key)`

### Commit `wait`
- writes: `audit_events`
- required fields:
  - `tenant_id`, `engine=PH1.X`, `event_type=Other`, `reason_code`, `correlation_id`, `turn_id`
  - payload: `directive=wait`, `wait_kind`, `work_order_id`, `work_order_status_snapshot`, `pending_state`
- idempotency_key rule (exact formula):
  - dedupe key = `(correlation_id, idempotency_key)`

## 5) Relations & Keys

FKs used by this slice:
- `audit_events.user_id -> identities.user_id` (nullable)
- `audit_events.device_id -> devices.device_id` (nullable)
- `audit_events.session_id -> sessions.session_id` (nullable)

Unique / dedupe constraints used by this slice:
- `audit_idempotency_index_legacy(correlation_id, idempotency_key)` in storage wiring

State/boundary constraints:
- No PH1.X-owned current table in row 16 scope.
- No PH1.X migration is required for this slice.
- PH1.X remains non-authoritative; storage scope is audit-only.
- PH1.X gating states are explicit and reason-coded:
  - `CLARIFY` (missing/ambiguous fields)
  - `CONFIRM` (impactful intent awaiting yes/no)
  - `DISPATCH` (candidate handoff only)
  - `RESPOND` / `WAIT` (non-executing output/control)
- PH1.X continuity gates are explicit and fail closed:
  - `subject_ref` is required on every PH1.X request and is stamped into returned thread state.
  - `active_speaker_user_id` is required on every PH1.X request and is stamped into returned thread state.
  - if `thread_state.active_speaker_user_id != request.active_speaker_user_id`: PH1.X emits a single clarify (`X_CONTINUITY_SPEAKER_MISMATCH`) and does not dispatch.
  - if `thread_state.active_subject_ref != request.subject_ref` while a pending state exists: PH1.X emits a single clarify (`X_CONTINUITY_SUBJECT_MISMATCH`) and does not dispatch.
- WorkOrder state transitions (`DRAFT -> CLARIFY -> CONFIRM -> EXECUTING`) are persisted by `SELENE_OS_CORE_TABLES`; PH1.X emits directive rows that deterministically wire back into those transitions.

## 6) Audit Emissions (PH1.J)

PH1.X writes emit PH1.J audit events with:
- `event_type`:
  - `XConfirm`
  - `XDispatch`
  - `Other` (for `clarify`, `respond`, `wait`)
- `reason_code(s)`:
  - deterministic PH1.X reason codes from the PH1.X contract output path
- `payload_min` keys (bounded):
  - `directive`
  - `confirm_kind`
  - `what_is_missing`
  - `clarification_unit_id`
  - `response_kind`
  - `dispatch_target`
  - `wait_kind`
  - `work_order_id`
  - `work_order_status_snapshot`
  - `pending_state`
  - `lease_token_hash`

## 7) Acceptance Tests (DB Wiring Proof)

- `AT-X-01` tenant isolation enforced
  - `at_x_db_01_tenant_isolation_enforced`
- `AT-X-02` append-only enforcement for PH1.X ledger writes
  - `at_x_db_02_append_only_enforced`
- `AT-X-03` idempotency dedupe works
  - `at_x_db_03_idempotency_dedupe_works`
- `AT-X-04` no PH1.X current-table rebuild is required
  - `at_x_db_04_no_current_table_rebuild_required`
- `AT-X-13` continuity speaker mismatch is fail-closed and clarify-only
  - `at_x_continuity_speaker_mismatch_fails_closed_into_one_clarify`
- `AT-X-14` continuity subject mismatch with pending state is fail-closed and clarify-only
  - `at_x_continuity_subject_mismatch_with_pending_fails_closed_into_one_clarify`

Implementation references:
- storage wiring: `crates/selene_storage/src/ph1f.rs`
- typed repo: `crates/selene_storage/src/repo.rs`
- migration: none required for row 16 (`PH1.X` uses existing `audit_events`)
- tests: `crates/selene_storage/tests/ph1_x/db_wiring.rs`

## 8) Related Engine Boundary (`PH1.PRUNE`)

- Before PH1.X `clarify` commit rows are emitted, Selene OS may invoke PH1.PRUNE when multiple required fields exist.
- If PH1.PRUNE is used, `what_is_missing` should map to PH1.PRUNE `selected_missing_field`; if PH1.PRUNE validation fails, Selene OS must fail closed and skip PRUNE-derived handoff.
- PH1.X remains authoritative for one conversational move per turn; PH1.PRUNE output is advisory input only.

## 9) Related Engine Boundary (`PH1.EXPLAIN`)

- PH1.X may request PH1.EXPLAIN packets only for explicit explain triggers (`why/how/what happened`) or accountability prompts.
- PH1.X decides whether to surface PH1.EXPLAIN output; PH1.EXPLAIN remains advisory and non-executing.
- PH1.X must never treat PH1.EXPLAIN text as authority, confirmation, or execution approval.

## 10) Related Engine Boundary (`PH1.EMO.GUIDE`)

- Before PH1.X final `respond` directive shaping, Selene OS may invoke PH1.EMO.GUIDE to compute a bounded style profile hint (`DOMINANT | GENTLE` + ordered modifiers).
- If PH1.EMO.GUIDE validation fails (`validation_status != OK`), Selene OS must fail closed on EMO.GUIDE handoff and continue without EMO-guided tone hints.
- PH1.X may consume PH1.EMO.GUIDE hints for tone/pacing only; PH1.X must not allow EMO.GUIDE to alter facts, missing-field logic, confirmation semantics, or dispatch gating.

## 11) Related Engine Boundary (`PH1.PERSONA`)

- Before PH1.X final `respond` directive shaping, Selene OS may invoke PH1.PERSONA and forward persona hints only when `PERSONA_PROFILE_VALIDATE` returns `validation_status=OK`.
- PH1.X may consume `style_profile_ref` + `delivery_policy_ref` as phrasing/tone posture hints only.
- PH1.X must not allow PH1.PERSONA output to alter truth, intent selection, missing-field logic, confirmation semantics, access outcomes, or dispatch gating.
- If PH1.PERSONA validation fails, Selene OS must fail closed on persona handoff and continue with deterministic default PH1.X behavior.
