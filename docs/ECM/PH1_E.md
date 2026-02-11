# PH1.E ECM Spec

## Engine Header
- `engine_id`: `PH1.E`
- `purpose`: Persist deterministic PH1.E read-only tool outcomes (`tool_ok`, `tool_fail`) as bounded audit rows.
- `data_owned`: `audit_events` writes in PH1.E scope
- `version`: `v1`
- `status`: `ACTIVE`

## Capability List

### `PH1E_TOOL_OK_COMMIT_ROW`
- `name`: Commit successful tool outcome
- `input_schema`: `(now, tenant_id, correlation_id, turn_id, session_id?, user_id, device_id, tool_name, query_hash, cache_status, reason_code, idempotency_key)`
- `output_schema`: `Result<AuditEventId, StorageError>`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `DECLARED (DB_WRITE)`

### `PH1E_TOOL_FAIL_COMMIT_ROW`
- `name`: Commit failed tool outcome
- `input_schema`: `(now, tenant_id, correlation_id, turn_id, session_id?, user_id, device_id, tool_name, fail_code, cache_status, reason_code, idempotency_key)`
- `output_schema`: `Result<AuditEventId, StorageError>`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `DECLARED (DB_WRITE)`

### `PH1E_READ_AUDIT_ROWS`
- `name`: Read PH1.E audit rows for one correlation thread
- `input_schema`: `correlation_id`
- `output_schema`: `AuditEvent[]`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `NONE`

## Failure Modes + Reason Codes
- PH1.E deterministic failure codes include:
  - `E_FAIL_FORBIDDEN_TOOL`
  - `E_FAIL_TIMEOUT`
  - `E_FAIL_BUDGET_EXCEEDED`
  - `E_FAIL_POLICY_BLOCK`
  - `E_FAIL_FORBIDDEN_DOMAIN`
- storage scope/idempotency violations are fail-closed and reason-coded.

## Audit Emission Requirements Per Capability
- write capabilities emit PH1.J with bounded keys:
  - `tool_name`
  - `query_hash`
  - `fail_code`
  - `cache_status`
- `PH1E_READ_AUDIT_ROWS` emits audit only in replay/diagnostic mode.

## Sources
- `crates/selene_storage/src/repo.rs` (`Ph1ERepo`)
- `docs/DB_WIRING/PH1_E.md`
