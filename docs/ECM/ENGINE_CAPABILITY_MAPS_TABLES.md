# Engine Capability Maps Tables ECM Spec

## Engine Header
- `engine_id`: `ENGINE_CAPABILITY_MAPS_TABLES`
- `purpose`: Persist engine capability-map event ledger and rebuild current callable capability bindings.
- `data_owned`: `engine_capability_maps`, `engine_capability_maps_current`
- `version`: `v1`
- `status`: `ACTIVE`

## Capability List

### `ECM_APPEND_ENGINE_CAPABILITY_MAP_ROW`
- `name`: Append engine capability-map event row
- `input_schema`: `EngineCapabilityMapEventInput`
- `output_schema`: `Result<engine_capability_map_event_id, StorageError>`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `DECLARED (DB_WRITE)`

### `ECM_REBUILD_CURRENT_ROWS`
- `name`: Rebuild current capability bindings from ledger
- `input_schema`: `none`
- `output_schema`: `in-memory projection updated`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `NONE`

### `ECM_READ_CURRENT_ROW`
- `name`: Read tenant+engine+capability binding
- `input_schema`: `(tenant_id, engine_id, capability_id)`
- `output_schema`: `Option<EngineCapabilityMapCurrentRecord>`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `NONE`

## Failure Modes + Reason Codes
- append-only mutation attempt: `ECM_APPEND_ONLY_VIOLATION`
- idempotency replay/no-op: `ECM_IDEMPOTENCY_REPLAY`
- tenant scope mismatch: `ECM_TENANT_SCOPE_VIOLATION`
- contract validation failure: `ECM_CONTRACT_VALIDATION_FAILED`

## Audit Emission Requirements Per Capability
- `ECM_APPEND_ENGINE_CAPABILITY_MAP_ROW` must emit PH1.J audit events with:
  - `event_type`
  - `reason_code`
  - `tenant_id`
  - `engine_id`
  - `capability_id`
  - `idempotency_key`
- Rebuild/read operations emit audit only in explicit governance/replay flows.

## Sources
- `crates/selene_storage/src/repo.rs` (`EngineCapabilityMapsTablesRepo`)
- `docs/DB_WIRING/ENGINE_CAPABILITY_MAPS_TABLES.md`
