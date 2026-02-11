# PH1.WRITE ECM Spec

## Engine Header
- `engine_id`: `PH1.WRITE`
- `purpose`: Persist deterministic formatting/presentation decisions for approved response text without changing meaning.
- `data_owned`: `audit_events` writes in PH1.WRITE scope
- `version`: `v1`
- `status`: `ACTIVE`

## Capability List

### `PH1WRITE_FORMAT_COMMIT_ROW`
- `name`: Commit PH1.WRITE formatting decision
- `input_schema`: `(now, tenant_id, correlation_id, turn_id, session_id?, user_id, device_id, format_mode, reason_code, idempotency_key)`
- `output_schema`: `Result<AuditEventId, StorageError>`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `DECLARED (DB_WRITE)`

### `PH1WRITE_READ_AUDIT_ROWS`
- `name`: Read PH1.WRITE audit rows for one correlation thread
- `input_schema`: `correlation_id`
- `output_schema`: `AuditEvent[]`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `NONE`

## Failure Modes + Reason Codes
- PH1.WRITE outputs must carry deterministic reason codes from formatting outcomes (including safe fallback-to-original mode).
- storage scope/idempotency failures are fail-closed and reason-coded.

## Audit Emission Requirements Per Capability
- `PH1WRITE_FORMAT_COMMIT_ROW` emits PH1.J event rows with bounded payload keys:
  - `directive=format`
  - `format_mode`
- `PH1WRITE_READ_AUDIT_ROWS` emits audit only in replay/diagnostic mode.

## Sources
- `crates/selene_storage/src/repo.rs` (`Ph1WriteRepo`)
- `docs/DB_WIRING/PH1_WRITE.md`
