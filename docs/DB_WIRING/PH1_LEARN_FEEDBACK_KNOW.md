# PH1.LEARN / PH1.FEEDBACK / PH1.KNOW DB Wiring Spec

## 1) Engine Header

- `engine_id`: `PH1.LEARN / PH1.FEEDBACK / PH1.KNOW`
- `purpose`: Persist learning-layer feedback signals and versioned adaptation/dictionary artifacts using existing append-only ledgers (`audit_events` + `artifacts_ledger`) with deterministic tenant scope and idempotency.
- `version`: `v1`
- `status`: `PASS`

## 2) Data Owned (authoritative)

### `audit_events` (PH1.FEEDBACK)
- `truth_type`: `LEDGER`
- `primary key`: `event_id`
- invariants:
  - PH1.FEEDBACK feedback signals are append-only (`engine=PH1.FEEDBACK`)
  - feedback payload is bounded structured JSON (`feedback_event_type`, `signal_bucket`)
  - idempotent retry dedupe applies via canonical audit idempotency rules

### `artifacts_ledger` (PH1.LEARN / PH1.KNOW)
- `truth_type`: `LEDGER`
- `primary key`: `artifact_id`
- invariants:
  - PH1.LEARN writes versioned adaptation artifacts (`created_by=PH1.LEARN`)
  - PH1.KNOW writes tenant dictionary packs (`created_by=PH1.KNOW`)
  - scope/version uniqueness is enforced by `(scope_type, scope_id, artifact_type, artifact_version)`
  - idempotent retries are deduped by `(scope_type, scope_id, artifact_type, artifact_version, idempotency_key)`
  - append-only enforcement applies (`no overwrite`, `no delete`)

No PH1.LEARN/PH1.FEEDBACK/PH1.KNOW-owned current projection table is introduced in row 25.

## 3) Reads (dependencies)

- identity/session scope checks for PH1.FEEDBACK writes:
  - reads: `identities`, `devices`, `sessions`
  - required checks:
    - `device_id` must belong to `user_id`
    - when provided, `session_id` must match `(user_id, device_id)`
    - one deterministic tenant binding per `device_id` in PH1.FEEDBACK rows
- scope checks for PH1.LEARN artifacts:
  - `Tenant` scope requires `scope_id == tenant_id`
  - `User` scope requires existing `identities.user_id` and deterministic user->tenant binding
  - `Device` scope is out of row-25 lock scope and is rejected
- PH1.KNOW dictionary pack constraints:
  - tenant scope only
  - `artifact_type` bounded to `STT_VOCAB_PACK | TTS_PRONUNCIATION_PACK`

## 4) Writes (outputs)

### `FEEDBACK_EVENT_COMMIT`
- writes: `audit_events`
- required fields:
  - `tenant_id`, `engine=PH1.FEEDBACK`, `reason_code`, `correlation_id`, `turn_id`, scoped user/device references
  - payload keys:
    - `feedback_event_type`
    - `signal_bucket`
- idempotency key rule:
  - dedupe key = canonical audit `(correlation_id, idempotency_key)` path

### `LEARN_ARTIFACT_COMMIT`
- writes: `artifacts_ledger`
- required fields:
  - `scope_type`, `scope_id`, `artifact_type`, `artifact_version`, `package_hash`, `payload_ref`, `provenance_ref`, `status`, `created_by=PH1.LEARN`
- idempotency key rule:
  - dedupe key = `(scope_type, scope_id, artifact_type, artifact_version, idempotency_key)`

### `KNOW_DICTIONARY_PACK_COMMIT`
- writes: `artifacts_ledger`
- required fields:
  - `scope_type=Tenant`, `scope_id=tenant_id`, `artifact_type in {STT_VOCAB_PACK, TTS_PRONUNCIATION_PACK}`, `artifact_version`, `package_hash`, `payload_ref`, `provenance_ref`, `created_by=PH1.KNOW`
- idempotency key rule:
  - dedupe key = `(Tenant, tenant_id, artifact_type, artifact_version, idempotency_key)`

## 5) Relations & Keys

- PH1.FEEDBACK FK checks:
  - `audit_events.user_id -> identities.user_id`
  - `audit_events.device_id -> devices.device_id`
  - optional `audit_events.session_id -> sessions.session_id`
- PH1.LEARN/PH1.KNOW uniqueness and replay keys:
  - `artifacts_ledger` unique scope/version index is authoritative
  - `artifacts_ledger` idempotency index is authoritative
- row-25 boundary:
  - no writes to `memory_*`, `access_*`, onboarding, reminder, or broadcast tables

## 6) Audit/Proof Emissions

Row 25 lock proof is ledger discipline:
- PH1.FEEDBACK emits reason-coded append-only audit rows
- PH1.LEARN/PH1.KNOW emit append-only artifact rows with deterministic scope/version + idempotency semantics
- no hidden current-state mutation path exists in this slice

## 7) Acceptance Tests (DB Wiring Proof)

- `AT-LEARNDB-01` tenant isolation enforced
  - `at_learn_db_01_tenant_isolation_enforced`
- `AT-LEARNDB-02` append-only enforced
  - `at_learn_db_02_append_only_enforced`
- `AT-LEARNDB-03` idempotency dedupe works
  - `at_learn_db_03_idempotency_dedupe_works`
- `AT-LEARNDB-04` ledger-only (no current-table rebuild in this row)
  - `at_learn_db_04_ledger_only_no_current_rebuild_required`

Implementation references:
- storage wiring: `crates/selene_storage/src/ph1f.rs`
- typed repo: `crates/selene_storage/src/repo.rs`
- migration: none required for row 25 (uses existing `audit_events` + `artifacts_ledger`)
- tests: `crates/selene_storage/tests/ph1_learn_feedback_know/db_wiring.rs`
