# PH1.WRITE DB Wiring Spec

## 1) Engine Header

- `engine_id`: `PH1.WRITE`
- `purpose`: Persist deterministic writing/formatting outcomes as bounded audit events without introducing PH1.WRITE-owned tables.
- `version`: `v1`
- `status`: `PASS`

## 2) Data Owned (authoritative)

### `os_core.audit_events`
- `truth_type`: `LEDGER`
- `primary key`: `event_id`
- invariants:
  - PH1.WRITE outcomes are recorded with `engine=Other("PH1.WRITE")`
  - `event_type=Other` with bounded payload keys
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
  - one tenant binding per `device_id` for PH1.WRITE rows
- why this read is required: fail closed before PH1.WRITE audit writes

### Replay reads
- reads: `audit_events` by `correlation_id` and `tenant_id`
- keys/joins used: deterministic correlation chain reads
- required indices:
  - `audit_events(correlation_id, turn_id)`
  - `audit_events(tenant_id, created_at)` (or equivalent tenant filter path)
- scope rules: no cross-tenant writes; tenant attribution required
- why this read is required: deterministic replay and dedupe verification

## 4) Writes (outputs)

### Commit `format`
- writes: `audit_events`
- required fields:
  - `tenant_id`, `engine=Other("PH1.WRITE")`, `event_type=Other`, `reason_code`, `correlation_id`, `turn_id`
  - payload: `directive=format`, `format_mode`
- idempotency_key rule (exact formula):
  - dedupe key = `(correlation_id, idempotency_key)`
- deterministic `format_mode` examples:
  - `FORMATTED_TEXT`
  - `FALLBACK_ORIGINAL`

## 5) Relations & Keys

FKs used by this slice:
- `audit_events.user_id -> identities.user_id` (nullable)
- `audit_events.device_id -> devices.device_id` (nullable)
- `audit_events.session_id -> sessions.session_id` (nullable)

Unique / dedupe constraints used by this slice:
- `audit_idempotency_index_legacy(correlation_id, idempotency_key)` in storage wiring

State/boundary constraints:
- No PH1.WRITE-owned current table in row 17 scope.
- No PH1.WRITE migration is required for this slice.
- PH1.WRITE is presentation-only; storage scope is audit-only.

## 6) Audit Emissions (PH1.J)

PH1.WRITE writes emit PH1.J audit events with:
- `event_type`:
  - `Other`
- `reason_code(s)`:
  - deterministic PH1.WRITE reason codes from formatting outcomes
- `payload_min` keys (bounded):
  - `directive`
  - `format_mode`

## 7) Acceptance Tests (DB Wiring Proof)

- `AT-WRITE-DB-01` tenant isolation enforced
  - `at_write_db_01_tenant_isolation_enforced`
- `AT-WRITE-DB-02` append-only enforcement for PH1.WRITE ledger writes
  - `at_write_db_02_append_only_enforced`
- `AT-WRITE-DB-03` idempotency dedupe works
  - `at_write_db_03_idempotency_dedupe_works`
- `AT-WRITE-DB-04` no PH1.WRITE current-table rebuild is required
  - `at_write_db_04_no_current_table_rebuild_required`

Implementation references:
- storage wiring: `crates/selene_storage/src/ph1f.rs`
- typed repo: `crates/selene_storage/src/repo.rs`
- migration: none required for row 17 (`PH1.WRITE` uses existing `audit_events`)
- tests: `crates/selene_storage/tests/ph1_write/db_wiring.rs`
