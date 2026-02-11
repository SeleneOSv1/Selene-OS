# PH1.PERSONA DB Wiring Spec

## 1) Engine Header

- `engine_id`: `PH1.PERSONA`
- `purpose`: Persist deterministic per-user personalization profile decisions (`style_profile_ref`, `delivery_policy_ref`, `preferences_snapshot_ref`) as bounded audit records without introducing a PH1.PERSONA-owned state table in MVP.
- `version`: `v1`
- `status`: `PASS`

## 2) Data Owned (authoritative)

### `os_core.audit_events`
- `truth_type`: `LEDGER`
- `primary key`: `event_id`
- invariants:
  - PH1.PERSONA outputs are recorded with `engine=PH1.PERSONA` (audit engine `Other("PH1.PERSONA")`)
  - payload is bounded and structured: `style_profile_ref`, `delivery_policy_ref`, `preferences_snapshot_ref`
  - idempotent append dedupe via `(correlation_id, idempotency_key)`
  - append-only enforcement applies (`no overwrite`, `no delete`)

No PH1.PERSONA-owned `current` table is introduced in this row.

## 3) Reads (dependencies)

- identity/device/session scope:
  - reads: `identities`, `devices`, `sessions`
  - required checks:
    - `device_id` must belong to `user_id`
    - when provided, `session_id` must match `(user_id, device_id)`
- persona context source:
  - reads: `memory_current`, `memory_ledger`, `conversation_ledger` (advisory context only; no direct writes by PH1.PERSONA)
- tenant isolation:
  - one deterministic tenant binding per `device_id` for PH1.PERSONA audit rows

## 4) Writes (outputs)

### `PERSONA_PROFILE_COMMIT`
- writes: `audit_events`
- required fields:
  - `tenant_id`, `engine=PH1.PERSONA`, `event_type=Other`, `reason_code`, `correlation_id`, `turn_id`
  - payload keys:
    - `style_profile_ref`
    - `delivery_policy_ref`
    - `preferences_snapshot_ref`
- idempotency key rule:
  - dedupe key = `(correlation_id, idempotency_key)`

## 5) Relations & Keys

FK checks used by this slice:
- `audit_events.user_id -> identities.user_id` (nullable in schema, required by PH1.PERSONA write path)
- `audit_events.device_id -> devices.device_id`
- `audit_events.session_id -> sessions.session_id` (optional path)

State/boundary constraints:
- PH1.PERSONA does not mutate `memory_*` tables directly in this row.
- PH1.PERSONA remains non-authoritative and cannot bypass Access/Simulation gates.

## 6) Audit/Proof Emissions

Row 24 lock proof is audit-ledger persistence discipline:
- deterministic tenant/device scope validation
- append-only audit writes
- idempotent dedupe on retry
- replayable PH1.PERSONA decision trace by `correlation_id`

## 7) Acceptance Tests (DB Wiring Proof)

- `AT-PERS-DB-01` tenant isolation enforced
  - `at_pers_db_01_tenant_isolation_enforced`
- `AT-PERS-DB-02` append-only enforced
  - `at_pers_db_02_append_only_enforced`
- `AT-PERS-DB-03` idempotency dedupe works
  - `at_pers_db_03_idempotency_dedupe_works`
- `AT-PERS-DB-04` no PH1.PERSONA current-table rebuild required
  - `at_pers_db_04_no_current_table_rebuild_required`

Implementation references:
- storage wiring: `crates/selene_storage/src/ph1f.rs`
- typed repo: `crates/selene_storage/src/repo.rs`
- migration: none required for row 24 (uses existing `audit_events`)
- tests: `crates/selene_storage/tests/ph1_persona/db_wiring.rs`
