# PH1.D DB Wiring Spec

## 1) Engine Header

- `engine_id`: `PH1.D`
- `purpose`: Persist deterministic PH1.D LLM-router outcomes (`chat`, `intent`, `clarify`, `analysis`, `fail_closed`) as bounded audit events without introducing PH1.D-owned tables.
- `version`: `v1`
- `status`: `PASS`

## 2) Data Owned (authoritative)

### `os_core.audit_events`
- `truth_type`: `LEDGER`
- `primary key`: `event_id`
- invariants:
  - PH1.D outcomes are recorded with `engine=PH1.D`
  - `event_type=Other` is used with explicit bounded payload keys (contract is carried by payload + `reason_code`)
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
  - one tenant binding per `device_id` for PH1.D rows
- why this read is required: fail closed before PH1.D audit writes

### Replay reads
- reads: `audit_events` by `correlation_id` and `tenant_id`
- keys/joins used: deterministic correlation chain reads
- required indices:
  - `audit_events(correlation_id, turn_id)`
  - `audit_events(tenant_id, created_at)` (or equivalent tenant filter path)
- scope rules: no cross-tenant writes; tenant attribution required
- why this read is required: deterministic replay and dedupe verification

## 4) Writes (outputs)

### Commit `chat`
- writes: `audit_events`
- required fields:
  - `tenant_id`, `engine=PH1.D`, `event_type=Other`, `reason_code`, `correlation_id`, `turn_id`
  - payload: `decision=CHAT`, `output_mode=chat`
- idempotency_key rule (exact formula):
  - dedupe key = `(correlation_id, idempotency_key)`
- failure reason codes (minimum examples):
  - `D_FAIL_INVALID_SCHEMA`
  - `D_FAIL_FORBIDDEN_OUTPUT`
  - `D_FAIL_TIMEOUT`

### Commit `intent`
- writes: `audit_events`
- required fields:
  - `tenant_id`, `engine=PH1.D`, `event_type=Other`, `reason_code`, `correlation_id`, `turn_id`
  - payload: `decision=INTENT`, `refined_intent_type`, `output_mode=intent`
- idempotency_key rule (exact formula):
  - dedupe key = `(correlation_id, idempotency_key)`
- failure reason codes (minimum examples):
  - `D_FAIL_INVALID_SCHEMA`
  - `D_FAIL_FORBIDDEN_OUTPUT`
  - `D_FAIL_BUDGET_EXCEEDED`

### Commit `clarify`
- writes: `audit_events`
- required fields:
  - `tenant_id`, `engine=PH1.D`, `event_type=Other`, `reason_code`, `correlation_id`, `turn_id`
  - payload: `decision=CLARIFY`, `what_is_missing`, `output_mode=clarify`
- idempotency_key rule (exact formula):
  - dedupe key = `(correlation_id, idempotency_key)`
- failure reason codes (minimum examples):
  - `D_FAIL_INVALID_SCHEMA`
  - `D_FAIL_FORBIDDEN_OUTPUT`

### Commit `analysis`
- writes: `audit_events`
- required fields:
  - `tenant_id`, `engine=PH1.D`, `event_type=Other`, `reason_code`, `correlation_id`, `turn_id`
  - payload: `decision=ANALYSIS`, `analysis_kind`, `output_mode=analysis`
- idempotency_key rule (exact formula):
  - dedupe key = `(correlation_id, idempotency_key)`
- failure reason codes (minimum examples):
  - `D_FAIL_INVALID_SCHEMA`

### Commit `fail_closed`
- writes: `audit_events`
- required fields:
  - `tenant_id`, `engine=PH1.D`, `event_type=Other`, `reason_code`, `correlation_id`, `turn_id`
  - payload: `decision=FAIL_CLOSED`, `fail_code`, `output_mode=fail`
- idempotency_key rule (exact formula):
  - dedupe key = `(correlation_id, idempotency_key)`
- failure reason codes (minimum examples):
  - `D_FAIL_INVALID_SCHEMA`
  - `D_FAIL_FORBIDDEN_OUTPUT`
  - `D_FAIL_TIMEOUT`
  - `D_FAIL_BUDGET_EXCEEDED`

## 5) Relations & Keys

FKs used by this slice:
- `audit_events.user_id -> identities.user_id` (nullable)
- `audit_events.device_id -> devices.device_id` (nullable)
- `audit_events.session_id -> sessions.session_id` (nullable)

Unique / dedupe constraints used by this slice:
- `audit_idempotency_index_legacy(correlation_id, idempotency_key)` in storage wiring

State/boundary constraints:
- No PH1.D-owned current table in row 15 scope.
- No PH1.D migration is required for this slice.
- PH1.D remains non-authoritative; storage scope is audit-only.

## 6) Audit Emissions (PH1.J)

PH1.D writes emit PH1.J audit events with:
- `event_type`:
  - `Other` (payload-bounded PH1.D decision contract)
- `reason_code(s)`:
  - deterministic PH1.D reason codes from the PH1.D contract output/failure path
- `payload_min` keys (bounded):
  - `decision`
  - `output_mode`
  - `refined_intent_type`
  - `what_is_missing`
  - `analysis_kind`
  - `fail_code`

## 7) Acceptance Tests (DB Wiring Proof)

- `AT-D-DB-01` tenant isolation enforced
  - `at_d_db_01_tenant_isolation_enforced`
- `AT-D-DB-02` append-only enforcement for PH1.D ledger writes
  - `at_d_db_02_append_only_enforced`
- `AT-D-DB-03` idempotency dedupe works
  - `at_d_db_03_idempotency_dedupe_works`
- `AT-D-DB-04` no PH1.D current-table rebuild is required
  - `at_d_db_04_no_current_table_rebuild_required`

Implementation references:
- storage wiring: `crates/selene_storage/src/ph1f.rs`
- typed repo: `crates/selene_storage/src/repo.rs`
- migration: none required for row 15 (`PH1.D` uses existing `audit_events`)
- tests: `crates/selene_storage/tests/ph1_d/db_wiring.rs`
