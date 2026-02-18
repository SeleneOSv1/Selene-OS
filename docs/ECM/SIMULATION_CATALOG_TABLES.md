# Simulation Catalog Tables ECM Spec

## Engine Header
- `engine_id`: `SIMULATION_CATALOG_TABLES`
- `purpose`: Persist simulation catalog event ledger and rebuild current ACTIVE/DRAFT/DEPRECATED simulation records.
- `data_owned`: `simulation_catalog`, `simulation_catalog_current`
- `version`: `v1`
- `status`: `ACTIVE`

## Capability List

### `SIMCAT_APPEND_SIMULATION_CATALOG_ROW`
- `name`: Append simulation catalog event row
- `input_schema`: `SimulationCatalogEventInput`
- `output_schema`: `Result<simulation_catalog_event_id, StorageError>`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `DECLARED (DB_WRITE)`

### `SIMCAT_REBUILD_CURRENT_ROWS`
- `name`: Rebuild simulation catalog current projection from ledger
- `input_schema`: `none`
- `output_schema`: `in-memory projection updated`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `NONE`

### `SIMCAT_READ_CURRENT_ROW`
- `name`: Read tenant-scoped simulation current row
- `input_schema`: `(tenant_id, simulation_id)`
- `output_schema`: `Option<SimulationCatalogCurrentRecord>`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `NONE`

## Failure Modes + Reason Codes
- append-only mutation attempt: `SIMCAT_APPEND_ONLY_VIOLATION`
- idempotency replay/no-op: `SIMCAT_IDEMPOTENCY_REPLAY`
- tenant scope mismatch: `SIMCAT_TENANT_SCOPE_VIOLATION`
- contract validation failure: `SIMCAT_CONTRACT_VALIDATION_FAILED`

## Audit Emission Requirements Per Capability
- `SIMCAT_APPEND_SIMULATION_CATALOG_ROW` must emit PH1.J audit events with:
  - `event_type`
  - `reason_code`
  - `tenant_id`
  - `simulation_id`
  - `idempotency_key`
- Rebuild/read operations emit audit only in explicit governance/replay flows.

## Related Governance Boundary
- `PH1.GOV` owns deterministic activation/deprecation/rollback decision logic for simulation definitions before `SIMCAT_APPEND_SIMULATION_CATALOG_ROW` transitions are allowed.

## Sources
- `crates/selene_storage/src/repo.rs` (`SimulationCatalogTablesRepo`)
- `docs/DB_WIRING/SIMULATION_CATALOG_TABLES.md`
