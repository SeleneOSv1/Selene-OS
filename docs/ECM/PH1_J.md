# PH1.J ECM Spec

## Engine Header
- `engine_id`: `PH1.J`
- `purpose`: Own canonical append-only audit envelope persistence and replay-safe audit reads.
- `data_owned`: `audit_events`
- `version`: `v1`
- `status`: `ACTIVE`

## Capability List

### `PH1J_APPEND_AUDIT_ROW`
- `name`: Append canonical audit event
- `input_schema`: `AuditEventInput`
- `output_schema`: `Result<AuditEventId, StorageError>`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `DECLARED (DB_WRITE)`

### `PH1J_AUDIT_ROWS_BY_CORRELATION`
- `name`: Read audit rows by correlation chain
- `input_schema`: `CorrelationId`
- `output_schema`: `Vec<AuditEvent>`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `NONE`

### `PH1J_AUDIT_ROWS_BY_TENANT`
- `name`: Read tenant-scoped audit rows
- `input_schema`: `tenant_id`
- `output_schema`: `Vec<AuditEvent>`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `NONE`

## Failure Modes + Reason Codes
- append-only mutation attempt: `J_APPEND_ONLY_VIOLATION`
- missing required reason_code/payload contract violation: `J_CONTRACT_VALIDATION_FAILED`
- scoped idempotency replay/no-op: `J_IDEMPOTENCY_REPLAY`
- tenant scope mismatch: `J_TENANT_SCOPE_VIOLATION`

## Audit Emission Requirements Per Capability
- `PH1J_APPEND_AUDIT_ROW` must persist required envelope fields:
  - `event_type`, `reason_code`, `severity`, `correlation_id`, `turn_id`, `payload_min`, `evidence_ref`, `idempotency_key`
- `payload_min` must use allowlisted bounded keys only.
- Reads do not emit new events by default.

## Sources
- `crates/selene_storage/src/repo.rs` (`Ph1jAuditRepo`)
- `docs/DB_WIRING/PH1_J.md`
