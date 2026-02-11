# PH1.M ECM Spec

## Engine Header
- `engine_id`: `PH1.M`
- `purpose`: Persist deterministic, consent-aware memory history and rebuildable current memory projection.
- `data_owned`: `memory_ledger`, `memory_current`
- `version`: `v1`
- `status`: `ACTIVE`

## Capability List

### `PH1M_APPEND_LEDGER_ROW`
- `name`: Append memory ledger event and project current memory state
- `input_schema`: `(user_id, memory_ledger_event, use_policy, expires_at?, idempotency_key?)`
- `output_schema`: `Result<ledger_id, StorageError>`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `DECLARED (DB_WRITE)`

### `PH1M_READ_MEMORY_LEDGER_ROWS`
- `name`: Read append-only memory ledger rows
- `input_schema`: `none`
- `output_schema`: `MemoryLedgerRow[]`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `NONE`

### `PH1M_READ_MEMORY_CURRENT_ROWS`
- `name`: Read full memory current projection map
- `input_schema`: `none`
- `output_schema`: `Map<(user_id, memory_key), MemoryCurrentRecord>`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `NONE`

### `PH1M_READ_MEMORY_CURRENT_ROW`
- `name`: Read one memory current row
- `input_schema`: `(user_id, memory_key)`
- `output_schema`: `Option<MemoryCurrentRecord>`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `NONE`

### `PH1M_REBUILD_CURRENT_FROM_LEDGER`
- `name`: Rebuild memory current projection from append-only memory ledger
- `input_schema`: `none`
- `output_schema`: `unit`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `DECLARED (DB_WRITE_CURRENT_PROJECTION)`

### `PH1M_APPEND_ONLY_GUARD`
- `name`: Guard against overwrite of memory ledger rows
- `input_schema`: `ledger_id`
- `output_schema`: `Result<(), StorageError>`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `NONE`

## Failure Modes + Reason Codes
- deterministic memory-policy failures (consent/use-policy violations) are fail-closed and reason-coded.
- storage invariants fail-closed on:
  - scope mismatch (`user_id` binding)
  - append-only violation
  - idempotency replay/no-op

## Audit Emission Requirements Per Capability
- write capabilities must emit PH1.J reason-coded events with bounded payload.
- rebuild/guard/read capabilities emit audit only in replay/diagnostic mode.

## Sources
- `crates/selene_storage/src/repo.rs` (`Ph1MRepo`)
- `docs/DB_WIRING/PH1_M.md`
