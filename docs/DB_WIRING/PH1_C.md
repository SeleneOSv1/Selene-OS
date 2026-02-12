# PH1.C DB Wiring Spec

## 1) Engine Header

- `engine_id`: `PH1.C`
- `purpose`: Persist deterministic STT gate outcomes without inventing new PH1.C tables by wiring transcript acceptance to `conversation_ledger` and STT decisions/metadata to `audit_events`.
- `version`: `v1`
- `status`: `PASS`

## 2) Data Owned (authoritative)

### `os_core.conversation_ledger`
- `truth_type`: `LEDGER`
- `primary key`: `conversation_turn_id`
- invariants:
  - FK `user_id -> identities.user_id`
  - optional FK `device_id -> devices.device_id`
  - optional FK `session_id -> sessions.session_id`
  - transcript commits use `source=VOICE_TRANSCRIPT`, `role=USER`
  - transcript status is represented deterministically:
    - accepted transcript -> `conversation_ledger` row present + `TranscriptOk` audit row
    - rejected transcript -> no `conversation_ledger` row + `TranscriptReject` audit row
  - idempotent append dedupe on `(correlation_id, idempotency_key)`
  - append-only; overwrite/delete prohibited

### `os_core.audit_events`
- `truth_type`: `LEDGER`
- `primary key`: `event_id`
- invariants:
  - STT gate outcomes are emitted as PH1.C audit rows (`TranscriptOk`, `TranscriptReject`, `SttCandidateEval`)
  - idempotent append dedupe on `(correlation_id, idempotency_key)` for this row scope
  - append-only; overwrite/delete prohibited

## 3) Reads (dependencies)

### Identity/device/session scope checks
- reads: `identities`, `devices`, `sessions`
- keys/joins used: direct FK existence lookups plus deterministic session scope check `(session.user_id, session.device_id)`
- required indices:
  - `identities(user_id)` (PK)
  - `devices(device_id)` (PK)
  - `sessions(session_id)` (PK)
- scope rules:
  - device must belong to `user_id`
  - one tenant binding per `device_id` in PH1.C runtime scope
- why this read is required: fail closed before transcript/audit persistence

### Transcript/audit replay reads
- reads:
  - `conversation_ledger` by `correlation_id`
  - `audit_events` by `correlation_id` and `tenant_id`
- keys/joins used: deterministic correlation threading
- required indices:
  - `conversation_ledger(correlation_id, turn_id)`
  - `audit_events(correlation_id, turn_id)`
- scope rules: no cross-tenant writes; tenant attribution is required on PH1.C audit rows
- why this read is required: deterministic replay and no duplicate transcript/audit rows on retries

## 4) Writes (outputs)

### Commit `transcript_ok`
- writes:
  - `conversation_ledger` (`VOICE_TRANSCRIPT` turn)
  - `audit_events` (`TranscriptOk` + `SttCandidateEval`)
- required fields:
  - transcript row: `created_at`, `correlation_id`, `turn_id`, `session_id?`, `user_id`, `device_id`, `text`, `text_hash`, `idempotency_key`
  - audit row: `tenant_id`, `engine=PH1.C`, `event_type`, `reason_code`, `correlation_id`, `turn_id`, payload, `evidence_ref?`
  - provider arbitration indicators (audit payload only): `route_class_used`, `attempt_count`, `candidate_count`, `selected_slot`, `mode_used`, `second_pass_used`
  - evidence span indicators (audit evidence_ref): `transcript_hash`, `critical_spans[]` with `start_byte`, `end_byte`, `field_hint?`
- idempotency_key rule (exact formula):
  - transcript dedupe key = `(correlation_id, idempotency_key)`
  - audit dedupe keys = `(correlation_id, idempotency_key + ":transcript_ok")` and `(correlation_id, idempotency_key + ":candidate_eval_ok")`
- failure reason codes (minimum examples):
  - `STT_FAIL_AUDIO_DEGRADED`
  - `STT_FAIL_LOW_CONFIDENCE`
  - `STT_FAIL_LANGUAGE_MISMATCH`

### Commit `transcript_reject`
- writes:
  - `audit_events` (`TranscriptReject` + `SttCandidateEval`)
- required fields:
  - reject row: `tenant_id`, `engine=PH1.C`, `event_type=TranscriptReject`, `reason_code`, `correlation_id`, `turn_id`, payload (`transcript_hash` optional)
  - candidate row: retry guidance metadata (`retry_advice`, `decision`) + arbitration indicators (`route_class_used`, `attempt_count`, `candidate_count`, `selected_slot`, `mode_used`, `second_pass_used`)
  - evidence span indicators (audit evidence_ref): `transcript_hash?`, `uncertain_spans[]` with byte offsets when available
- idempotency_key rule (exact formula):
  - audit dedupe keys = `(correlation_id, idempotency_key + ":transcript_reject")` and `(correlation_id, idempotency_key + ":candidate_eval_reject")`
- failure reason codes (minimum examples):
  - `STT_FAIL_EMPTY`
  - `STT_FAIL_LOW_COVERAGE`
  - `STT_FAIL_BUDGET_EXCEEDED`

## 5) Relations & Keys

FKs used by this slice:
- `conversation_ledger.user_id -> identities.user_id`
- `conversation_ledger.device_id -> devices.device_id` (nullable)
- `conversation_ledger.session_id -> sessions.session_id` (nullable)
- `audit_events.user_id -> identities.user_id` (nullable)
- `audit_events.device_id -> devices.device_id` (nullable)
- `audit_events.session_id -> sessions.session_id` (nullable)

Unique / dedupe constraints used by this slice:
- `conversation_idempotency_index(correlation_id, idempotency_key)`
- `audit_idempotency_index_legacy(correlation_id, idempotency_key)`

State/boundary constraints:
- No PH1.C-owned current table in row 13 scope.
- No PH1.C migration is required for this slice.
- PH1.C persistence remains ledger-only over existing core tables.

## 6) Audit Emissions (PH1.J)

PH1.C writes emit PH1.J audit events with:
- `event_type`:
  - `TranscriptOk`
  - `TranscriptReject`
  - `SttCandidateEval`
- `reason_code(s)`:
  - pass/reject reason codes from PH1.C
  - deterministic candidate-eval reason codes for replay bucketing
- `payload_min` allowlisted discipline:
  - `TranscriptOk` / `TranscriptReject`: `transcript_hash` only
  - `SttCandidateEval`: bounded metadata (`decision`, `language_tag`, `confidence_bucket`, `retry_advice`, `route_class_used`, `attempt_count`, `candidate_count`, `selected_slot`, `mode_used`, `second_pass_used`)
  - `evidence_ref` bounded structure (when present): `transcript_hash`, `spans[]` (`start_byte`, `end_byte`, `field_hint?`)

## 7) Acceptance Tests (DB Wiring Proof)

- `AT-PH1-C-DB-01` tenant isolation enforced
  - `at_c_db_01_tenant_isolation_enforced`
- `AT-PH1-C-DB-02` append-only enforcement for transcript/audit ledgers
  - `at_c_db_02_append_only_enforced`
- `AT-PH1-C-DB-03` idempotency dedupe works
  - `at_c_db_03_idempotency_dedupe_works`
- `AT-PH1-C-DB-04` no PH1.C current-table rebuild is required
  - `at_c_db_04_no_current_table_rebuild_required`

Implementation references:
- storage wiring: `crates/selene_storage/src/ph1f.rs`
- typed repo: `crates/selene_storage/src/repo.rs`
- migration: none required for row 13 (`PH1.C` uses existing `conversation_ledger` + `audit_events`)
- tests: `crates/selene_storage/tests/ph1_c/db_wiring.rs`
