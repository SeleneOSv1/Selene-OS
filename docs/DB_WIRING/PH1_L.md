# PH1.L DB Wiring Spec

## 1) Engine Header

- `engine_id`: `PH1.L`
- `purpose`: Persist deterministic session lifecycle state (`OPEN | ACTIVE | SOFT_CLOSED | CLOSED | SUSPENDED`) in `sessions` with idempotent transition writes.
- `version`: `v1`
- `status`: `PASS`

## 2) Data Owned (authoritative)

### `os_core.sessions`
- `truth_type`: `CURRENT`
- `primary key`: `session_id`
- invariants:
  - FK `user_id -> identities.user_id`
  - FK `device_id -> devices.device_id`
  - for existing `session_id`: `user_id`, `device_id`, and `opened_at` are immutable
  - `last_activity_at` is monotonic per `session_id`
  - `closed_at` is required iff `session_state = CLOSED`
  - transition edge must be PH1.L-allowed (`OPEN/ACTIVE/SOFT_CLOSED/CLOSED/SUSPENDED` deterministic graph)

## 3) Reads (dependencies)

### Session lookup/read paths
- reads: `sessions(session_id, user_id, device_id, session_state, opened_at, last_activity_at, closed_at)`
- keys/joins: direct PK lookup by `session_id`; filtered reads by `user_id`/`device_id` + state for lifecycle checks
- required indices:
  - `sessions(session_id)` (PK)
  - `ix_sessions_user_state_last_activity`
  - `ix_sessions_device_state_last_activity`
  - `ix_sessions_state_last_activity`
- scope rules: user/device session scope (tenant isolation follows identity/device scoping discipline)
- why: PH1.L must load/update the current session snapshot deterministically and avoid cross-user leakage

### FK validation paths
- reads: `identities.user_id`, `devices.device_id`
- keys/joins: direct FK existence lookup before session write
- required indices: PK on `identities.user_id`, PK on `devices.device_id`
- scope rules: user/device scoped
- why: no orphan session rows

## 4) Writes (outputs)

### Upsert session lifecycle row
- writes:
  - `sessions(session_id, user_id, device_id, session_state, opened_at, last_activity_at, closed_at)`
- required fields:
  - `session_id`, `user_id`, `device_id`, `session_state`, `opened_at`, `last_activity_at`
  - `closed_at` when `session_state=CLOSED`
- idempotency_key rule:
  - deterministic caller-provided key
  - dedupe scope in PH1.F write path: `(session_id, idempotency_key)`
  - duplicate attempts return deterministic no-op/current row
- failure reason classes:
  - FK violation (`sessions.user_id`, `sessions.device_id`)
  - invalid transition edge
  - immutable field mutation attempt
  - invalid closed_at discipline

## 5) Relations & Keys

FKs:
- `sessions.user_id -> identities.user_id`
- `sessions.device_id -> devices.device_id`

Unique constraints:
- `sessions(session_id)` (PK)

State machine constraints:
- allowed PH1.L transitions enforced in storage write path
- no silent session identity rebinding (`user_id`/`device_id` immutable per `session_id`)

## 6) Audit Emissions (PH1.J)

PH1.L lifecycle transitions must emit PH1.J audit events with:
- `event_type`: `SESSION_OPEN | SESSION_SOFT_CLOSE | SESSION_CLOSED | SYSTEM_SUSPEND | SYSTEM_RESUME | STATE_TRANSITION`
- `reason_code(s)`: `L_OPEN_WAKE`, `L_RESUME_WAKE_SOFT_CLOSE`, `L_TO_SOFT_CLOSE_SILENCE`, `L_TO_CLOSED_SILENCE`, `L_TO_CLOSED_DISMISS`, `L_WAIT_TIMEOUT_PROMPTED`, `L_CLOSE_CHECK_PROMPTED`, `L_RESUME_USER_ACTIVITY`, `L_SUSPEND_AUDIO_DEGRADED`, `L_RESUME_STABLE`
- `payload_min` allowlisted keys: `state_from`, `state_to` (for `STATE_TRANSITION`), plus bounded context keys per event policy
- `evidence_ref`: optional, reference-only (no raw sensitive text/audio)

## 7) Acceptance Tests (DB Wiring Proof)

- `AT-L-DB-01` tenant isolation enforced
  - `at_l_db_01_tenant_isolation_enforced`
- `AT-L-DB-02` append-only enforcement (ledger path)
  - `at_l_db_02_append_only_enforced`
- `AT-L-DB-03` idempotency dedupe works
  - `at_l_db_03_idempotency_dedupe_works`
- `AT-L-DB-04` current-table proof for PH1.L scope (no PH1.L-owned session ledger in this slice)
  - `at_l_db_04_current_table_no_ledger_rebuild_required`

Implementation references:
- storage wiring: `crates/selene_storage/src/ph1f.rs`
- typed repo: `crates/selene_storage/src/repo.rs`
- migration: `crates/selene_storage/migrations/0007_ph1l_sessions_indexes.sql`
- tests: `crates/selene_storage/tests/ph1_l/db_wiring.rs`
