# PH1.LEARN / PH1.FEEDBACK / PH1.KNOW DB Wiring Spec (Storage Grouping Only)

## 1) Engine Header

- `engine_id`: `PH1.LEARN / PH1.FEEDBACK / PH1.KNOW`
- `purpose`: Persist learning-layer feedback signals and versioned adaptation/dictionary artifacts using existing append-only ledgers (`audit_events` + `artifacts_ledger`) with deterministic tenant scope and idempotency.
- `version`: `v1`
- `status`: `PASS`
- hard boundary:
  - this file defines persistence grouping semantics only.
  - this file is not a callable runtime engine contract.
  - runtime ownership stays split across `PH1.FEEDBACK`, `PH1.LEARN`, and `PH1.KNOW`.

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
  - PH1.LEARN writes versioned adaptation artifacts only (`created_by=PH1.LEARN`):
    - `STT_ROUTING_POLICY_PACK`
    - `STT_ADAPTATION_PROFILE`
    - `TTS_ROUTING_POLICY_PACK`
  - PH1.KNOW writes tenant dictionary artifacts only (`created_by=PH1.KNOW`):
    - `STT_VOCAB_PACK`
    - `TTS_PRONUNCIATION_PACK`
  - artifact-type ownership is single-writer and fail-closed:
    - PH1.LEARN must not write PH1.KNOW artifact types
    - PH1.KNOW must not write PH1.LEARN artifact types
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
- artifact_type boundary:
  - allowed only `STT_ROUTING_POLICY_PACK | STT_ADAPTATION_PROFILE | TTS_ROUTING_POLICY_PACK`
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
- `AT-LEARNDB-05` single-writer artifact ownership enforced
  - `at_learn_db_05_single_writer_artifact_types_enforced`

Implementation references:
- storage wiring: `crates/selene_storage/src/ph1f.rs`
- typed repo: `crates/selene_storage/src/repo.rs`
- migration: none required for row 25 (uses existing `audit_events` + `artifacts_ledger`)
- tests: `crates/selene_storage/tests/ph1_learn_feedback_know/db_wiring.rs`

## 8) Related Engine Boundary (`PH1.FEEDBACK`)

- Runtime FEEDBACK capability flow is defined in:
  - `docs/DB_WIRING/PH1_FEEDBACK.md`
  - `docs/ECM/PH1_FEEDBACK.md`
- This combined storage contract remains authoritative for append-only ledger persistence of feedback/artifact rows.

## 9) Related Engine Boundary (`PH1.LEARN`)

- Runtime LEARN capability flow is defined in:
  - `docs/DB_WIRING/PH1_LEARN.md`
  - `docs/ECM/PH1_LEARN.md`
- This combined storage contract remains authoritative for append-only artifact persistence semantics only (`created_by=PH1.LEARN`).

## 10) Related Engine Boundary (`PH1.KG`)

- PH1.KNOW tenant dictionary artifacts may be consumed by PH1.KG only as tenant-scoped seed metadata through Selene OS wiring.
- PH1.LEARN/PH1.KNOW storage rows must not be interpreted by PH1.KG as authority truth; PH1.KG still requires explicit evidence-backed, no-guessing relationship composition.

## 11) Related Engine Boundary (`PH1.KNOW`)

- Runtime PH1.KNOW capability flow is defined in:
  - `docs/DB_WIRING/PH1_KNOW.md`
  - `docs/ECM/PH1_KNOW.md`
- This combined row-25 storage contract remains authoritative for append-only artifact persistence semantics only (`created_by=PH1.KNOW`).
