# PH1.L ECM Spec

## Engine Header
- `engine_id`: `PH1.L`
- `purpose`: Persist deterministic session lifecycle transitions and read current session state under strict idempotency.
- `data_owned`: `sessions` (owned write scope in PH1.F)
- `version`: `v1`
- `status`: `ACTIVE`

## Capability List

### `PH1L_UPSERT_SESSION_LIFECYCLE_ROW`
- `name`: Upsert session lifecycle row with idempotency guard
- `input_schema`: `(SessionRecord, idempotency_key)`
- `output_schema`: `Result<SessionId, StorageError>`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `DECLARED (DB_WRITE)`

### `PH1L_READ_SESSION_ROW`
- `name`: Read one session row by `session_id`
- `input_schema`: `SessionId`
- `output_schema`: `Option<SessionRecord>`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `NONE`

### `PH1L_READ_SESSION_ROWS`
- `name`: Read full session map for deterministic replay/testing
- `input_schema`: `none`
- `output_schema`: `Map<SessionId, SessionRecord>`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `NONE`

## Failure Modes + Reason Codes
- session contract validation failure: `L_CONTRACT_VALIDATION_FAILED`
- idempotency replay/no-op: `L_IDEMPOTENCY_REPLAY`
- scope mismatch (tenant/user/device): `L_SCOPE_VIOLATION`

## Audit Emission Requirements Per Capability
- `PH1L_UPSERT_SESSION_LIFECYCLE_ROW` must emit PH1.J events with:
  - `event_type`: `STATE_TRANSITION`
  - `reason_code`
  - `session_id`
  - `correlation_id`
  - `turn_id`
  - `idempotency_key` when present
- Read capabilities emit audit only in explicit replay/diagnostic mode.

## Sources
- `crates/selene_storage/src/repo.rs` (`Ph1lSessionLifecycleRepo`)
- `docs/DB_WIRING/PH1_L.md`
