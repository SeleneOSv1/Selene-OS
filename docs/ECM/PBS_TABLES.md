# PBS Tables ECM Spec

## Engine Header
- `engine_id`: `PBS_TABLES`
- `purpose`: Persist process blueprint ledger events and rebuild deterministic blueprint registry current view.
- `data_owned`: `process_blueprints`, `blueprint_registry`
- `version`: `v1`
- `status`: `ACTIVE`

## Capability List

### `PBS_APPEND_PROCESS_BLUEPRINT_ROW`
- `name`: Append process blueprint event row
- `input_schema`: `ProcessBlueprintEventInput`
- `output_schema`: `Result<process_blueprint_event_id, StorageError>`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `DECLARED (DB_WRITE)`

### `PBS_REBUILD_BLUEPRINT_REGISTRY_ROWS`
- `name`: Rebuild blueprint registry current rows from process blueprint ledger
- `input_schema`: `none`
- `output_schema`: `in-memory projection updated`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `NONE`

### `PBS_READ_BLUEPRINT_REGISTRY_ROW`
- `name`: Read tenant+intent blueprint mapping
- `input_schema`: `(tenant_id, intent_type)`
- `output_schema`: `Option<BlueprintRegistryRecord>`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `NONE`

## Failure Modes + Reason Codes
- append-only mutation attempt: `PBS_APPEND_ONLY_VIOLATION`
- idempotency replay/no-op: `PBS_IDEMPOTENCY_REPLAY`
- tenant scope mismatch: `PBS_TENANT_SCOPE_VIOLATION`
- contract validation failure: `PBS_CONTRACT_VALIDATION_FAILED`

## Audit Emission Requirements Per Capability
- `PBS_APPEND_PROCESS_BLUEPRINT_ROW` must emit PH1.J audit events with:
  - `event_type`
  - `reason_code`
  - `tenant_id`
  - `idempotency_key`
- Rebuild/read operations emit audit only when executed under explicit diagnostic/replay mode.

## Sources
- `crates/selene_storage/src/repo.rs` (`PbsTablesRepo`)
- `docs/DB_WIRING/PBS_TABLES.md`
