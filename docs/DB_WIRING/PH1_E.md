# PH1.E DB Wiring Spec

## 1) Engine Header

- `engine_id`: `PH1.E`
- `purpose`: Persist deterministic PH1.E tool outcomes (`tool_ok`, `tool_fail`) as bounded audit events without introducing PH1.E-owned tables.
- `version`: `v1`
- `status`: `PASS`

## 2) Data Owned (authoritative)

### `os_core.audit_events`
- `truth_type`: `LEDGER`
- `primary key`: `event_id`
- invariants:
  - PH1.E outcomes are recorded with `engine=PH1.E`
  - event types used: `ToolOk`, `ToolFail`
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
  - one tenant binding per `device_id` for PH1.E rows
- why this read is required: fail closed before PH1.E audit writes

### Replay reads
- reads: `audit_events` by `correlation_id` and `tenant_id`
- keys/joins used: deterministic correlation chain reads
- required indices:
  - `audit_events(correlation_id, turn_id)`
  - `audit_events(tenant_id, created_at)` (or equivalent tenant filter path)
- scope rules: no cross-tenant writes; tenant attribution required
- why this read is required: deterministic replay and dedupe verification

## 4) Writes (outputs)

### Commit `tool_ok`
- writes: `audit_events`
- required fields:
  - `tenant_id`, `engine=PH1.E`, `event_type=ToolOk`, `reason_code`, `correlation_id`, `turn_id`
  - payload: `tool_name`, `query_hash`, `cache_status`
- idempotency_key rule (exact formula):
  - dedupe key = `(correlation_id, idempotency_key)`

### Commit `tool_fail`
- writes: `audit_events`
- required fields:
  - `tenant_id`, `engine=PH1.E`, `event_type=ToolFail`, `reason_code`, `correlation_id`, `turn_id`
  - payload: `tool_name`, `fail_code`, `cache_status`
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
- No PH1.E-owned current table in row 19 scope.
- No PH1.E migration is required for this slice.
- PH1.E remains read-only tooling; storage scope is audit-only.

## 6) Audit Emissions (PH1.J)

PH1.E writes emit PH1.J audit events with:
- `event_type`:
  - `ToolOk`
  - `ToolFail`
- `reason_code(s)`:
  - deterministic PH1.E reason codes from tool outcome paths
- `payload_min` keys (bounded):
  - `tool_name`
  - `query_hash`
  - `fail_code`
  - `cache_status`

## 7) Acceptance Tests (DB Wiring Proof)

- `AT-E-DB-01` tenant isolation enforced
  - `at_e_db_01_tenant_isolation_enforced`
- `AT-E-DB-02` append-only enforcement for PH1.E ledger writes
  - `at_e_db_02_append_only_enforced`
- `AT-E-DB-03` idempotency dedupe works
  - `at_e_db_03_idempotency_dedupe_works`
- `AT-E-DB-04` no PH1.E current-table rebuild is required
  - `at_e_db_04_no_current_table_rebuild_required`

Implementation references:
- storage wiring: `crates/selene_storage/src/ph1f.rs`
- typed repo: `crates/selene_storage/src/repo.rs`
- migration: none required for row 19 (`PH1.E` uses existing `audit_events`)
- tests: `crates/selene_storage/tests/ph1_e/db_wiring.rs`
