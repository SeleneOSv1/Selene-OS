# Selene OS Core Tables ECM Spec

## Engine Header
- `engine_id`: `SELENE_OS_CORE_TABLES`
- `purpose`: Persist and project WorkOrder ledger/current state under strict tenant scope and idempotency rules.
- `data_owned`: `work_order_ledger`, `work_orders_current`, `work_order_step_attempts`, `work_order_leases`
- `version`: `v1`
- `status`: `ACTIVE`

## Capability List

### `OS_WORK_ORDER_LEDGER_APPEND`
- `name`: Append work order ledger event
- `input_schema`: `WorkOrderLedgerEventInput`
- `output_schema`: `Result<work_order_event_id, StorageError>`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `DECLARED (DB_WRITE)`

### `OS_WORK_ORDER_CURRENT_READ`
- `name`: Read work order current state
- `input_schema`: `(tenant_id, work_order_id)`
- `output_schema`: `Option<WorkOrderCurrentRecord>`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `NONE`

### `OS_WORK_ORDER_LEDGER_READ`
- `name`: Read work order ledger stream
- `input_schema`: `tenant_id / work_order_id selectors`
- `output_schema`: `Vec<WorkOrderLedgerEvent>`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `NONE`

### `OS_WORK_ORDER_CURRENT_REBUILD`
- `name`: Rebuild work order current projection from ledger
- `input_schema`: `none`
- `output_schema`: `in-memory projection updated`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `NONE`

## Failure Modes + Reason Codes
- append-only mutation attempt: `OS_WORK_ORDER_APPEND_ONLY_VIOLATION`
- idempotency replay/no-op: `OS_WORK_ORDER_IDEMPOTENCY_REPLAY`
- tenant scope mismatch: `OS_WORK_ORDER_TENANT_SCOPE_VIOLATION`
- contract validation failure: `OS_WORK_ORDER_CONTRACT_VALIDATION_FAILED`

## Audit Emission Requirements Per Capability
- `OS_WORK_ORDER_LEDGER_APPEND` must emit PH1.J events containing:
  - `tenant_id`, `work_order_id`, `correlation_id`, `turn_id`, `event_type`, `reason_code`, `idempotency_key`
- Reads/rebuild operations should emit audit only when invoked as explicit replay/diagnostic operations.

## Sources
- `crates/selene_storage/src/repo.rs` (`SeleneOsWorkOrderRepo`)
- `docs/DB_WIRING/SELENE_OS_CORE_TABLES.md`
