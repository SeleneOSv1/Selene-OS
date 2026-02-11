# PH1.F ECM Spec

## Engine Header
- `engine_id`: `PH1.F`
- `purpose`: Provide deterministic persistence primitives for identity/device/session, memory ledger/current, and conversation ledger.
- `data_owned`: `identities`, `devices`, `sessions`, `memory_ledger`, `memory_current`, `conversation_ledger`
- `version`: `v1`
- `status`: `ACTIVE`

## Capability List

### `PH1F_INSERT_IDENTITY_ROW`
- `name`: Insert identity row
- `input_schema`: `IdentityRecord`
- `output_schema`: `Result<(), StorageError>`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `DECLARED (DB_WRITE)`

### `PH1F_INSERT_DEVICE_ROW`
- `name`: Insert device row
- `input_schema`: `DeviceRecord`
- `output_schema`: `Result<(), StorageError>`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `DECLARED (DB_WRITE)`

### `PH1F_INSERT_SESSION_ROW`
- `name`: Insert session row
- `input_schema`: `SessionRecord`
- `output_schema`: `Result<(), StorageError>`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `DECLARED (DB_WRITE)`

### `PH1F_APPEND_MEMORY_ROW`
- `name`: Append memory ledger event and update current projection
- `input_schema`: `(user_id, MemoryLedgerEvent, MemoryUsePolicy, expires_at, idempotency_key)`
- `output_schema`: `Result<ledger_id, StorageError>`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `DECLARED (DB_WRITE)`

### `PH1F_APPEND_CONVERSATION_ROW`
- `name`: Append conversation ledger turn
- `input_schema`: `ConversationTurnInput`
- `output_schema`: `Result<ConversationTurnId, StorageError>`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `DECLARED (DB_WRITE)`

### `PH1F_REBUILD_MEMORY_CURRENT_ROWS`
- `name`: Rebuild memory current projection from ledger
- `input_schema`: `none`
- `output_schema`: `in-memory projection updated`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `NONE`

### `PH1F_READ_FOUNDATION_STATE`
- `name`: Read identity/memory/conversation foundation state
- `input_schema`: `user_id / key selectors`
- `output_schema`: `identity row + memory rows + conversation rows`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `NONE`

## Failure Modes + Reason Codes
- FK failure (identity/device/session missing): `F_FK_SCOPE_VIOLATION`
- append-only mutation attempt: `F_APPEND_ONLY_VIOLATION`
- idempotency replay/no-op: `F_IDEMPOTENCY_REPLAY`
- contract validation failure: `F_CONTRACT_VALIDATION_FAILED`
- tenant scope mismatch: `F_TENANT_SCOPE_VIOLATION`

## Audit Emission Requirements Per Capability
- All write capabilities must emit PH1.J audit events with:
  - `event_type`
  - `reason_code`
  - `correlation_id`
  - `turn_id`
  - `idempotency_key` (when used)
- Required minimum events:
  - `PH1F_INSERT_*`: `STATE_TRANSITION`
  - `PH1F_APPEND_MEMORY_ROW`: `MEMORY_STORED | MEMORY_FORGOTTEN`
  - `PH1F_APPEND_CONVERSATION_ROW`: `CONVERSATION_TURN_STORED`

## Sources
- `crates/selene_storage/src/repo.rs` (`Ph1fFoundationRepo`)
- `docs/DB_WIRING/PH1_F.md`
