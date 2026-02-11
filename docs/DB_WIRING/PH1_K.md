# PH1.K DB Wiring Spec

## 1) Engine Header

- `engine_id`: `PH1.K`
- `purpose`: Persist deterministic voice-runtime substrate outputs (stream refs, device state, timing, interrupt candidates, degradation flags) as an append-only ledger plus a rebuildable current projection.
- `version`: `v1`
- `status`: `PASS`

## 2) Data Owned (authoritative)

### `os_core.audio_runtime_events`
- `truth_type`: `LEDGER`
- `primary key`: `event_id`
- invariants:
  - FK `device_id -> devices.device_id`
  - optional FK `session_id -> sessions.session_id`
  - `event_kind` in `STREAM_REFS | VAD_EVENT | DEVICE_STATE | TIMING_STATS | INTERRUPT_CANDIDATE | DEGRADATION_FLAGS | TTS_PLAYBACK_ACTIVE`
  - `device_health` in `HEALTHY | DEGRADED | FAILED` when present
  - idempotent append dedupe on `(tenant_id, device_id, event_kind, idempotency_key)`
  - append-only; overwrite/delete prohibited
  - one tenant binding per device in PH1.K runtime scope

### `os_core.audio_runtime_current`
- `truth_type`: `CURRENT` (materialized)
- `primary key`: `(tenant_id, device_id)`
- invariants:
  - FK `device_id -> devices.device_id`
  - optional FK `session_id -> sessions.session_id`
  - FK `last_event_id -> audio_runtime_events.event_id`
  - state is rebuildable from `audio_runtime_events` in deterministic event-id order
  - `device_health` in `HEALTHY | DEGRADED | FAILED` when present

## 3) Reads (dependencies)

### Device/session FK checks
- reads: `devices.device_id`, `sessions.session_id` (optional)
- keys/joins used: direct FK existence lookups
- required indices:
  - `devices(device_id)` (PK)
  - `sessions(session_id)` (PK)
- scope rules: PH1.K runtime writes are device-scoped and tenant-bound
- why this read is required: fail closed before runtime event persistence

### Runtime state reads
- reads:
  - `audio_runtime_events` by `(tenant_id, device_id)` and event id
  - `audio_runtime_current` by `(tenant_id, device_id)`
- keys/joins used: deterministic key lookups and ordered replay
- required indices:
  - `ux_audio_runtime_events_dedupe`
  - `ix_audio_runtime_events_tenant_device_time`
  - `audio_runtime_current(tenant_id, device_id)` (PK)
- scope rules: no cross-tenant reads for one device binding
- why this read is required: deterministic idempotency and current-state rebuild

## 4) Writes (outputs)

### Append PH1.K runtime event
- writes: `audio_runtime_events`
- required fields:
  - `tenant_id`, `device_id`, `event_kind`, `created_at`, `idempotency_key`
  - plus event-kind fields:
    - `STREAM_REFS`: `processed_stream_id`, `pre_roll_buffer_id`
    - `DEVICE_STATE`: `selected_mic`, `selected_speaker`, `device_health`
    - `TIMING_STATS`: `jitter_ms`, `drift_ppm`, `buffer_depth_ms`, `underruns`, `overruns`
    - `INTERRUPT_CANDIDATE`: `phrase_id`, `phrase_text`, `reason_code`
    - `DEGRADATION_FLAGS`: `capture_degraded`, `aec_unstable`, `device_changed`, `stream_gap_detected`
    - `TTS_PLAYBACK_ACTIVE`: `tts_playback_active`
- ledger event_type (if ledger): `K_RUNTIME_EVENT_COMMIT`
- idempotency_key rule (exact formula):
  - dedupe key = `(tenant_id, device_id, event_kind, idempotency_key)`
- failure reason codes (minimum examples):
  - `K_FAIL_DEVICE_UNBOUND`
  - `K_FAIL_SESSION_INVALID`
  - `K_FAIL_TENANT_SCOPE_MISMATCH`
  - `K_FAIL_EVENT_FIELDS_INVALID`

### Materialize/update PH1.K current runtime state
- writes: `audio_runtime_current`
- required fields:
  - `(tenant_id, device_id)`, `last_event_id`, `updated_at`
  - event-derived current fields (stream refs, device state, timing, interrupt/degradation snapshots)
- ledger event_type (if ledger): n/a (current projection update)
- idempotency_key rule (exact formula):
  - driven by source ledger dedupe key
- failure reason codes (minimum examples):
  - `K_FAIL_REBUILD_INTEGRITY`

## 5) Relations & Keys

FKs:
- `audio_runtime_events.device_id -> devices.device_id`
- `audio_runtime_events.session_id -> sessions.session_id` (nullable)
- `audio_runtime_current.device_id -> devices.device_id`
- `audio_runtime_current.session_id -> sessions.session_id` (nullable)
- `audio_runtime_current.last_event_id -> audio_runtime_events.event_id`

Unique constraints:
- `audio_runtime_events(event_id)` (PK)
- `ux_audio_runtime_events_dedupe`
- `audio_runtime_current(tenant_id, device_id)` (PK)

State/boundary constraints:
- `audio_runtime_events` is append-only.
- `audio_runtime_current` must be derivable from `audio_runtime_events` only.
- PH1.K persists substrate facts only; it does not persist intent/authority decisions.

## 6) Audit Emissions (PH1.J)

PH1.K runtime writes must emit PH1.J audit events with:
- `event_type`:
  - `K_STREAM_REFS_COMMIT`
  - `K_DEVICE_STATE_COMMIT`
  - `K_TIMING_STATS_COMMIT`
  - `K_INTERRUPT_CANDIDATE_COMMIT`
  - `K_DEGRADATION_FLAGS_COMMIT`
  - `K_TTS_PLAYBACK_ACTIVE_COMMIT`
- `reason_code(s)`:
  - `K_INTERRUPT_CANDIDATE_EMITTED`
  - `K_STREAM_GAP_DETECTED`
  - `K_AEC_UNSTABLE`
  - `K_DEVICE_FAILOVER`
  - `K_DEVICE_UNHEALTHY`
- `payload_min` allowlisted keys:
  - `tenant_id`
  - `device_id`
  - `session_id`
  - `event_kind`
  - `processed_stream_id`
  - `pre_roll_buffer_id`
  - `device_health`
  - `tts_playback_active`
  - `interrupt_phrase_id`
  - `interrupt_phrase_text`
  - `degradation_flags`

## 7) Acceptance Tests (DB Wiring Proof)

- `AT-K-DB-01` tenant isolation enforced
  - `at_k_db_01_tenant_isolation_enforced`
- `AT-K-DB-02` append-only enforcement for PH1.K runtime ledger
  - `at_k_db_02_append_only_enforced`
- `AT-K-DB-03` idempotency dedupe works
  - `at_k_db_03_idempotency_dedupe_works`
- `AT-K-DB-04` current-table rebuild from runtime ledger is deterministic
  - `at_k_db_04_current_table_rebuild_from_ledger`

Implementation references:
- storage wiring: `crates/selene_storage/src/ph1f.rs`
- typed repo: `crates/selene_storage/src/repo.rs`
- migration: `crates/selene_storage/migrations/0010_ph1k_audio_runtime_tables.sql`
- tests: `crates/selene_storage/tests/ph1_k/db_wiring.rs`
