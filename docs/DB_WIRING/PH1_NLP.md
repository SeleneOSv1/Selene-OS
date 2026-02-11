# PH1.NLP DB Wiring Spec

## 1) Engine Header

- `engine_id`: `PH1.NLP`
- `purpose`: Persist deterministic NLP normalization outcomes (`intent_draft`, `clarify`, `chat`) as bounded audit events without introducing new PH1.NLP-owned tables.
- `version`: `v1`
- `status`: `PASS`

## 2) Data Owned (authoritative)

### `os_core.audit_events`
- `truth_type`: `LEDGER`
- `primary key`: `event_id`
- invariants:
  - NLP outcomes are recorded with `engine=PH1.NLP`
  - event types used: `NlpIntentDraft`, `NlpClarify`
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
  - one tenant binding per `device_id` for PH1.NLP rows
- why this read is required: fail closed before NLP audit writes

### Replay reads
- reads: `audit_events` by `correlation_id` and `tenant_id`
- keys/joins used: deterministic correlation chain reads
- required indices:
  - `audit_events(correlation_id, turn_id)`
  - `audit_events(tenant_id, created_at)` (or equivalent tenant filter path)
- scope rules: no cross-tenant writes; tenant attribution required
- why this read is required: deterministic replay and dedupe verification

## 4) Writes (outputs)

### Commit `intent_draft`
- writes: `audit_events`
- required fields:
  - `tenant_id`, `engine=PH1.NLP`, `event_type=NlpIntentDraft`, `reason_code`, `correlation_id`, `turn_id`
  - payload: `decision=INTENT_DRAFT`, `intent_type`, `overall_confidence`
- idempotency_key rule (exact formula):
  - dedupe key = `(correlation_id, idempotency_key)`
- failure reason codes (minimum examples):
  - `NLP_INTENT_OK`
  - `NLP_INTENT_UNKNOWN`
  - `NLP_MULTI_INTENT`

### Commit `clarify`
- writes: `audit_events`
- required fields:
  - `tenant_id`, `engine=PH1.NLP`, `event_type=NlpClarify`, `reason_code`, `correlation_id`, `turn_id`
  - payload: `decision=CLARIFY`, `what_is_missing`
- idempotency_key rule (exact formula):
  - dedupe key = `(correlation_id, idempotency_key)`
- failure reason codes (minimum examples):
  - `NLP_CLARIFY_MISSING_FIELD`
  - `NLP_CLARIFY_AMBIGUOUS_REFERENCE`
  - `NLP_UNCERTAIN_SPAN`

### Commit `chat`
- writes: `audit_events`
- required fields:
  - `tenant_id`, `engine=PH1.NLP`, `event_type=NlpIntentDraft`, `reason_code`, `correlation_id`, `turn_id`
  - payload: `decision=CHAT`
- idempotency_key rule (exact formula):
  - dedupe key = `(correlation_id, idempotency_key)`
- failure reason codes (minimum examples):
  - `NLP_CHAT_DEFAULT`

## 5) Relations & Keys

FKs used by this slice:
- `audit_events.user_id -> identities.user_id` (nullable)
- `audit_events.device_id -> devices.device_id` (nullable)
- `audit_events.session_id -> sessions.session_id` (nullable)

Unique / dedupe constraints used by this slice:
- `audit_idempotency_index_legacy(correlation_id, idempotency_key)` in storage wiring

State/boundary constraints:
- No PH1.NLP-owned current table in row 14 scope.
- No PH1.NLP migration is required for this slice.
- PH1.NLP remains non-authoritative; storage scope is audit-only.

## 6) Audit Emissions (PH1.J)

PH1.NLP writes emit PH1.J audit events with:
- `event_type`:
  - `NlpIntentDraft`
  - `NlpClarify`
- `reason_code(s)`:
  - deterministic NLP reason codes from PH1.NLP contract output
- `payload_min` keys (bounded):
  - `decision`
  - `intent_type`
  - `overall_confidence`
  - `what_is_missing`

## 7) Acceptance Tests (DB Wiring Proof)

- `AT-NLP-DB-01` tenant isolation enforced
  - `at_nlp_db_01_tenant_isolation_enforced`
- `AT-NLP-DB-02` append-only enforcement for NLP ledger writes
  - `at_nlp_db_02_append_only_enforced`
- `AT-NLP-DB-03` idempotency dedupe works
  - `at_nlp_db_03_idempotency_dedupe_works`
- `AT-NLP-DB-04` no PH1.NLP current-table rebuild is required
  - `at_nlp_db_04_no_current_table_rebuild_required`

Implementation references:
- storage wiring: `crates/selene_storage/src/ph1f.rs`
- typed repo: `crates/selene_storage/src/repo.rs`
- migration: none required for row 14 (`PH1.NLP` uses existing `audit_events`)
- tests: `crates/selene_storage/tests/ph1_nlp/db_wiring.rs`
