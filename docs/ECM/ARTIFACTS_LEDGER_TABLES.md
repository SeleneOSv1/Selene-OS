# Artifacts Ledger Tables ECM Spec

## Engine Header
- `engine_id`: `ARTIFACTS_LEDGER_TABLES`
- `purpose`: Persist append-only artifact ledger entries and deterministic tool-cache rows for read-only acceleration.
- `data_owned`: `artifacts_ledger`, `tool_cache`
- `version`: `v1`
- `status`: `ACTIVE`

## Capability List

### `ART_APPEND_ARTIFACT_LEDGER_ROW`
- `name`: Append artifact ledger row
- `input_schema`: `ArtifactLedgerRowInput`
- `output_schema`: `Result<artifact_ledger_event_id, StorageError>`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `DECLARED (DB_WRITE)`

### `ART_READ_ARTIFACT_LEDGER_ROW`
- `name`: Read artifact ledger row by scope/type/version
- `input_schema`: `(scope_type, scope_id, artifact_type, artifact_version)`
- `output_schema`: `Option<ArtifactLedgerRow>`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `NONE`

### `ART_UPSERT_TOOL_CACHE_ROW`
- `name`: Upsert deterministic tool cache row
- `input_schema`: `ToolCacheRowInput`
- `output_schema`: `Result<tool_cache_row_id, StorageError>`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `DECLARED (DB_WRITE)`

### `ART_READ_TOOL_CACHE_ROW`
- `name`: Read tool cache row by deterministic key
- `input_schema`: `(tool_name, query_hash, locale)`
- `output_schema`: `Option<ToolCacheRow>`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `NONE`

## Failure Modes + Reason Codes
- append-only mutation attempt on ledger: `ART_APPEND_ONLY_VIOLATION`
- idempotency replay/no-op: `ART_IDEMPOTENCY_REPLAY`
- tenant/scope mismatch: `ART_SCOPE_VIOLATION`
- contract validation failure: `ART_CONTRACT_VALIDATION_FAILED`

## Audit Emission Requirements Per Capability
- `ART_APPEND_ARTIFACT_LEDGER_ROW` and `ART_UPSERT_TOOL_CACHE_ROW` must emit PH1.J audit events with:
  - `event_type`
  - `reason_code`
  - `tenant_id` or scoped owner id
  - `idempotency_key` when present
- Read capabilities emit audit only in explicit diagnostics/replay flows.

## Sources
- `crates/selene_storage/src/repo.rs` (`ArtifactsLedgerTablesRepo`)
- `docs/DB_WIRING/ARTIFACTS_LEDGER_TABLES.md`
