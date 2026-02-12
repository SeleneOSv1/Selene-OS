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
- `output_schema`: `Option<WorkOrderCurrentRecord>` including continuity fields:
  - `asked_fields_json`
  - `resolved_fields_json`
  - `prompt_dedupe_keys_json`
  - `external_approval_pending`
  - `external_approval_request_id`
  - `external_approval_target_user_id`
  - `external_approval_expires_at`
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

### `WORKORDER_LIST_PENDING`
- `name`: List pending work orders for continuity resume offers
- `input_schema`: `(tenant_id, requester_user_id, status_set={DRAFT|CLARIFY|CONFIRM}, recency_window, top_n)`
- `output_schema`: `Vec<PendingWorkOrderRecord>`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `NONE`

### `WORKORDER_CANCEL`
- `name`: Cancel one pending work order with append-only status event
- `input_schema`: `(tenant_id, work_order_id, reason_code, idempotency_key)`
- `output_schema`: `(work_order_id, status=CANCELED, last_event_id)`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `DECLARED (DB_WRITE)`

## Continuity + No-Repeat Rules
- Selene OS uses `asked_fields_json` and `prompt_dedupe_keys_json` to enforce global never-ask-twice behavior.
- If `external_approval_pending=true`, Selene OS should enter wait posture (single wait notice, no repeated prompts) until state changes.

## Failure Modes + Reason Codes
- append-only mutation attempt: `OS_WORK_ORDER_APPEND_ONLY_VIOLATION`
- idempotency replay/no-op: `OS_WORK_ORDER_IDEMPOTENCY_REPLAY`
- tenant scope mismatch: `OS_WORK_ORDER_TENANT_SCOPE_VIOLATION`
- contract validation failure: `OS_WORK_ORDER_CONTRACT_VALIDATION_FAILED`
- invalid cancel transition: `OS_WORK_ORDER_CANCEL_INVALID_STATE`

## Audit Emission Requirements Per Capability
- `OS_WORK_ORDER_LEDGER_APPEND` must emit PH1.J events containing:
  - `tenant_id`, `work_order_id`, `correlation_id`, `turn_id`, `event_type`, `reason_code`, `idempotency_key`
- `WORKORDER_CANCEL` must emit PH1.J event with:
  - `event_type=WORK_ORDER_CANCELED` (or `STATUS_CHANGED` to `CANCELED`), deterministic `reason_code`, and idempotency key
- Reads/rebuild operations should emit audit only when invoked as explicit replay/diagnostic operations.

## Sources
- `crates/selene_storage/src/repo.rs` (`SeleneOsWorkOrderRepo`)
- `docs/DB_WIRING/SELENE_OS_CORE_TABLES.md`
