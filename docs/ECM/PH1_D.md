# PH1.D ECM Spec

## Engine Header
- `engine_id`: `PH1.D`
- `purpose`: Persist deterministic PH1.D model-boundary outputs (`chat`, `intent`, `clarify`, `analysis`, `fail_closed`) as bounded audit rows.
- `data_owned`: `audit_events` writes in PH1.D scope
- `version`: `v1`
- `status`: `ACTIVE`

## Capability List

### `PH1D_CHAT_COMMIT_ROW`
- `name`: Commit PH1.D chat output decision
- `input_schema`: `(now, tenant_id, correlation_id, turn_id, session_id?, user_id, device_id, reason_code, idempotency_key)`
- `output_schema`: `Result<AuditEventId, StorageError>`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `DECLARED (DB_WRITE)`

### `PH1D_INTENT_COMMIT_ROW`
- `name`: Commit PH1.D intent refinement decision
- `input_schema`: `(now, tenant_id, correlation_id, turn_id, session_id?, user_id, device_id, refined_intent_type, reason_code, idempotency_key)`
- `output_schema`: `Result<AuditEventId, StorageError>`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `DECLARED (DB_WRITE)`

### `PH1D_CLARIFY_COMMIT_ROW`
- `name`: Commit PH1.D clarify decision
- `input_schema`: `(now, tenant_id, correlation_id, turn_id, session_id?, user_id, device_id, what_is_missing, reason_code, idempotency_key)`
- `output_schema`: `Result<AuditEventId, StorageError>`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `DECLARED (DB_WRITE)`

### `PH1D_ANALYSIS_COMMIT_ROW`
- `name`: Commit PH1.D analysis decision
- `input_schema`: `(now, tenant_id, correlation_id, turn_id, session_id?, user_id, device_id, analysis_kind, reason_code, idempotency_key)`
- `output_schema`: `Result<AuditEventId, StorageError>`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `DECLARED (DB_WRITE)`

### `PH1D_FAIL_CLOSED_COMMIT_ROW`
- `name`: Commit PH1.D fail-closed decision
- `input_schema`: `(now, tenant_id, correlation_id, turn_id, session_id?, user_id, device_id, fail_code, reason_code, idempotency_key)`
- `output_schema`: `Result<AuditEventId, StorageError>`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `DECLARED (DB_WRITE)`

### `PH1D_READ_AUDIT_ROWS`
- `name`: Read PH1.D audit rows for one correlation thread
- `input_schema`: `correlation_id`
- `output_schema`: `AuditEvent[]`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `NONE`

## Failure Modes + Reason Codes
- PH1.D deterministic failure reason codes:
  - `D_FAIL_INVALID_SCHEMA`
  - `D_FAIL_FORBIDDEN_OUTPUT`
  - `D_FAIL_SAFETY_BLOCK`
  - `D_FAIL_TIMEOUT`
  - `D_FAIL_BUDGET_EXCEEDED`
- every non-fail output is still reason-coded; scope/contract violations fail closed.

## Audit Emission Requirements Per Capability
- write capabilities emit PH1.J rows with bounded keys only:
  - `decision`
  - `output_mode`
  - `refined_intent_type`
  - `what_is_missing`
  - `analysis_kind`
  - `fail_code`
- read capability emits audit only in explicit replay/diagnostic mode.

## Sources
- `crates/selene_storage/src/repo.rs` (`Ph1dRouterRepo`)
- `docs/DB_WIRING/PH1_D.md`
