# PH1.K ECM Spec

## Engine Header
- `engine_id`: `PH1.K`
- `purpose`: Persist deterministic voice runtime substrate facts (stream refs, device/timing/interruption/degradation signals) as append-only events plus rebuildable current state.
- `data_owned`: `audio_runtime_events`, `audio_runtime_current`, `conversation_ledger` (PH1.K VAD markers only)
- `version`: `v1`
- `status`: `ACTIVE`

## Capability List

### `PH1K_RUNTIME_EVENT_COMMIT_ROW`
- `name`: Commit one PH1.K runtime event row and project current state
- `input_schema`: `(now, tenant_id, device_id, session_id?, event_kind, stream/device/timing/interrupt/degradation fields, idempotency_key)`
- `output_schema`: `Result<Ph1kRuntimeEventRecord, StorageError>`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `DECLARED (DB_WRITE)`

### `PH1K_VAD_MARKER_COMMIT_ROW`
- `name`: Commit one bounded PH1.K VAD marker row to conversation ledger
- `input_schema`: `(now, correlation_id, turn_id, session_id?, user_id, device_id?, vad_state, idempotency_key)`
- `output_schema`: `Result<ConversationTurnId, StorageError>`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `DECLARED (DB_WRITE)`

### `PH1K_READ_RUNTIME_EVENT_ROWS`
- `name`: Read PH1.K runtime ledger rows
- `input_schema`: `none`
- `output_schema`: `Ph1kRuntimeEventRecord[]`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `NONE`

### `PH1K_READ_RUNTIME_CURRENT_ROWS`
- `name`: Read PH1.K current projection map
- `input_schema`: `none`
- `output_schema`: `Map<(tenant_id, device_id), Ph1kRuntimeCurrentRecord>`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `NONE`

### `PH1K_READ_RUNTIME_CURRENT_ROW`
- `name`: Read one `(tenant_id, device_id)` current projection row
- `input_schema`: `(tenant_id, device_id)`
- `output_schema`: `Option<Ph1kRuntimeCurrentRecord>`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `NONE`

### `PH1K_REBUILD_RUNTIME_CURRENT_ROWS`
- `name`: Rebuild PH1.K current projection from append-only ledger
- `input_schema`: `none`
- `output_schema`: `unit`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `DECLARED (DB_WRITE_CURRENT_PROJECTION)`

### `PH1K_APPEND_ONLY_GUARD`
- `name`: Guard against overwrite of PH1.K runtime ledger rows
- `input_schema`: `event_id`
- `output_schema`: `Result<(), StorageError>`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `NONE`

## Failure Modes + Reason Codes
- contract/field validation failure: `K_FAIL_EVENT_FIELDS_INVALID`
- invalid or missing scope binding: `K_FAIL_TENANT_SCOPE_MISMATCH`
- invalid session reference: `K_FAIL_SESSION_INVALID`
- device binding failure: `K_FAIL_DEVICE_UNBOUND`
- invalid VAD marker scope/content: `K_FAIL_VAD_MARKER_SCOPE_INVALID`
- idempotency replay/no-op: `K_IDEMPOTENCY_REPLAY`

## Audit Emission Requirements Per Capability
- `PH1K_RUNTIME_EVENT_COMMIT_ROW` must emit PH1.J with bounded payload and reason code for each event class:
  - `K_STREAM_REFS_COMMIT`
  - `K_DEVICE_STATE_COMMIT`
  - `K_TIMING_STATS_COMMIT`
  - `K_INTERRUPT_CANDIDATE_COMMIT`
  - `K_VAD_EVENT_MARKER_COMMIT`
  - `K_DEGRADATION_FLAGS_COMMIT`
  - `K_TTS_PLAYBACK_ACTIVE_COMMIT`
- `PH1K_REBUILD_RUNTIME_CURRENT_ROWS` emits audit only in replay/diagnostic mode.
- Read/guard capabilities emit audit only when explicitly run under verification traces.

## Sources
- `crates/selene_storage/src/repo.rs` (`Ph1kVoiceRuntimeRepo`)
- `docs/DB_WIRING/PH1_K.md`
