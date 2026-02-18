# PH1.TTS DB Wiring Spec

## 1) Engine Header

- `engine_id`: `PH1.TTS`
- `purpose`: Persist deterministic TTS runtime outcomes (`tts_render_summary`, `tts_started`, `tts_canceled`, `tts_failed`) as bounded audit events without introducing PH1.TTS-owned tables.
- `version`: `v1`
- `status`: `PASS`

## 2) Data Owned (authoritative)

### `os_core.audit_events`
- `truth_type`: `LEDGER`
- `primary key`: `event_id`
- invariants:
  - PH1.TTS outcomes are recorded with `engine=PH1.Tts`
  - event types used: `TtsRenderSummary`, `TtsStarted`, `TtsCanceled`, `TtsFailed`
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
  - one tenant binding per `device_id` for PH1.TTS rows
- why this read is required: fail closed before PH1.TTS audit writes

### Replay reads
- reads: `audit_events` by `correlation_id` and `tenant_id`
- keys/joins used: deterministic correlation chain reads
- required indices:
  - `audit_events(correlation_id, turn_id)`
  - `audit_events(tenant_id, created_at)` (or equivalent tenant filter path)
- scope rules: no cross-tenant writes; tenant attribution required
- why this read is required: deterministic replay and dedupe verification

### Pronunciation assist inputs (related engine boundary)
- reads: bounded pronunciation pack hints supplied by Selene OS from `PH1.PRON`
- keys/joins used: `pack_id` + tenant scope match (input-only contract)
- scope rules:
  - pronunciation hints must remain tenant-scoped
  - user-scoped pronunciation hints require explicit consent proof before use
- why this read is required: pronunciation shaping for TTS output without changing semantic content

## 4) Writes (outputs)

### Commit `tts_render_summary`
- writes: `audit_events`
- required fields:
  - `tenant_id`, `engine=PH1.Tts`, `event_type=TtsRenderSummary`, `reason_code`, `correlation_id`, `turn_id`
  - payload: `route_class_used`, `mode_used`, `voice_id`
- idempotency_key rule (exact formula):
  - dedupe key = `(correlation_id, idempotency_key)`

### Commit `tts_started`
- writes: `audit_events`
- required fields:
  - `tenant_id`, `engine=PH1.Tts`, `event_type=TtsStarted`, `reason_code`, `correlation_id`, `turn_id`
  - payload: `answer_id`
- idempotency_key rule (exact formula):
  - dedupe key = `(correlation_id, idempotency_key)`

### Commit `tts_canceled`
- writes: `audit_events`
- required fields:
  - `tenant_id`, `engine=PH1.Tts`, `event_type=TtsCanceled`, `reason_code`, `correlation_id`, `turn_id`
  - payload: `answer_id`, `stop_reason`
- idempotency_key rule (exact formula):
  - dedupe key = `(correlation_id, idempotency_key)`

### Commit `tts_failed`
- writes: `audit_events`
- required fields:
  - `tenant_id`, `engine=PH1.Tts`, `event_type=TtsFailed`, `reason_code`, `correlation_id`, `turn_id`
  - payload: `answer_id`, `fail_code`
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
- No PH1.TTS-owned current table in row 18 scope.
- No PH1.TTS migration is required for this slice.
- PH1.TTS remains rendering-only; storage scope is audit-only.

## 6) Audit Emissions (PH1.J)

PH1.TTS writes emit PH1.J audit events with:
- `event_type`:
  - `TtsRenderSummary`
  - `TtsStarted`
  - `TtsCanceled`
  - `TtsFailed`
- `reason_code(s)`:
  - deterministic PH1.TTS reason codes from runtime outcomes
- `payload_min` keys (bounded):
  - `route_class_used`
  - `mode_used`
  - `voice_id`
  - `answer_id`
  - `stop_reason`
  - `fail_code`

## 7) Acceptance Tests (DB Wiring Proof)

- `AT-TTS-01` tenant isolation enforced
  - `at_tts_db_01_tenant_isolation_enforced`
- `AT-TTS-02` append-only enforcement for PH1.TTS ledger writes
  - `at_tts_db_02_append_only_enforced`
- `AT-TTS-03` idempotency dedupe works
  - `at_tts_db_03_idempotency_dedupe_works`
- `AT-TTS-04` no PH1.TTS current-table rebuild is required
  - `at_tts_db_04_no_current_table_rebuild_required`

Implementation references:
- storage wiring: `crates/selene_storage/src/ph1f.rs`
- typed repo: `crates/selene_storage/src/repo.rs`
- migration: none required for row 18 (`PH1.TTS` uses existing `audit_events`)
- tests: `crates/selene_storage/tests/ph1_tts/db_wiring.rs`

## 8) Related Engine Boundary (`PH1.KNOW`)

- Selene OS may provide PH1.KNOW pronunciation-hint subsets to PH1.TTS as optional rendering hints.
- PH1.TTS must treat PH1.KNOW output as advisory only and must not alter semantic meaning.
- PH1.KNOW hints must remain tenant-scoped and authorized-only before PH1.TTS consumption.

## 9) Related Engine Boundary (`PH1.EMO.GUIDE`)

- Selene OS may provide PH1.EMO.GUIDE style-profile hints (`DOMINANT | GENTLE` + ordered modifiers) to PH1.TTS render-plan inputs only after EMO.GUIDE validation passes.
- PH1.TTS must treat PH1.EMO.GUIDE output as tone/render policy only and must not alter factual meaning, intent outcomes, or execution semantics.
- If PH1.EMO.GUIDE is unavailable or fails validation, PH1.TTS must fail open on style hint input (use deterministic default render plan) while preserving all core safety gates.

## 10) Related Engine Boundary (`PH1.PERSONA`)

- Selene OS may provide PH1.PERSONA hints (`style_profile_ref`, `delivery_policy_ref`) to PH1.TTS render-plan inputs only after `PERSONA_PROFILE_VALIDATE` returns `validation_status=OK`.
- PH1.TTS must treat PH1.PERSONA output as advisory rendering posture only and must not alter factual meaning, intent outcomes, confirmation semantics, or execution order.
- If PH1.PERSONA is unavailable or fails validation, PH1.TTS must use deterministic default render policy while preserving all core safety gates.
