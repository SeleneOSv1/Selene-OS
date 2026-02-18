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
  - payload: `decision=INTENT_DRAFT`, `intent_type`, `overall_confidence`, `required_fields`, `ambiguity_flags`
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
  - payload: `decision=CLARIFY`, `what_is_missing`, `clarification_unit_id`, `accepted_answer_formats`
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
  - `required_fields`
  - `ambiguity_flags`
  - `what_is_missing`
  - `clarification_unit_id`

## 7) Acceptance Tests (DB Wiring Proof)

- `AT-PH1-NLP-DB-01` tenant isolation enforced
  - `at_nlp_db_01_tenant_isolation_enforced`
- `AT-PH1-NLP-DB-02` append-only enforcement for NLP ledger writes
  - `at_nlp_db_02_append_only_enforced`
- `AT-PH1-NLP-DB-03` idempotency dedupe works
  - `at_nlp_db_03_idempotency_dedupe_works`
- `AT-PH1-NLP-DB-04` no PH1.NLP current-table rebuild is required
  - `at_nlp_db_04_no_current_table_rebuild_required`

Implementation references:
- storage wiring: `crates/selene_storage/src/ph1f.rs`
- typed repo: `crates/selene_storage/src/repo.rs`
- migration: none required for row 14 (`PH1.NLP` uses existing `audit_events`)
- tests: `crates/selene_storage/tests/ph1_nlp/db_wiring.rs`

## 8) Related Engine Boundary (`PH1.PRUNE`)

- When `required_fields_missing` contains multiple fields, Selene OS may invoke `PH1.PRUNE` before PH1.X clarify.
- PH1.NLP remains the source of truth for missing field keys; PH1.PRUNE must not add keys not present in PH1.NLP output.
- PH1.NLP persistence model is unchanged: audit rows remain PH1.NLP-owned writes only.

## 8A) Related Engine Boundary (`PH1.SRL`)

- PH1.SRL is the deterministic post-STT repair layer immediately upstream of PH1.NLP.
- PH1.NLP may consume SRL repaired transcript/frame metadata only through Selene OS wiring.
- PH1.NLP remains authoritative for final `intent_draft | clarify | chat` outputs; SRL never grants execution/authority outcomes.

## 9) Related Engine Boundary (salience ranking)

- Before PH1.NLP parse/normalization finalization, Selene OS may pass bounded salience/focus metadata derived from deterministic upstream context handling.
- PH1.NLP remains authoritative for intent/clarify/chat outputs; salience hints are advisory only.
- PH1.NLP audit payloads may include focus-span references only as bounded metadata and must remain deterministic.

## 10) Related Engine Boundary (tangled utterance parsing)

- When utterance structure remains tangled after SRL, PH1.NLP handles deterministic unravel + clarify flow internally.
- PH1.NLP remains the only deterministic owner of final `intent_draft | clarify | chat` outputs.
- If ambiguity remains after internal unravel, PH1.NLP must preserve the no-guess rule and keep missing/ambiguous fields explicit.

## 11) Related Engine Boundary (`PH1.KNOW`)

- Selene OS may pass PH1.KNOW dictionary hints into PH1.NLP as advisory context only.
- PH1.NLP remains authoritative for deterministic `intent_draft | clarify | chat` outputs and must not treat PH1.KNOW hints as evidence replacement.
- PH1.KNOW hints must stay tenant-scoped and authorized-only; unknown/unverified terms still require clarify when confidence is not HIGH.
