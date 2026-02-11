# PH1.PERSONA ECM Spec

## Engine Header
- `engine_id`: `PH1.PERSONA`
- `purpose`: Persist deterministic personalization profile decisions as bounded audit records.
- `data_owned`: `audit_events` writes in PH1.PERSONA scope
- `version`: `v1`
- `status`: `ACTIVE`

## Capability List

### `PH1PERSONA_PROFILE_COMMIT_ROW`
- `name`: Commit persona profile decision snapshot
- `input_schema`: `(now, tenant_id, correlation_id, turn_id, session_id?, user_id, device_id, style_profile_ref, delivery_policy_ref, preferences_snapshot_ref, reason_code, idempotency_key)`
- `output_schema`: `Result<AuditEventId, StorageError>`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `DECLARED (DB_WRITE)`

### `PH1PERSONA_READ_AUDIT_ROWS`
- `name`: Read persona audit rows for one correlation thread
- `input_schema`: `correlation_id`
- `output_schema`: `AuditEvent[]`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `NONE`

## Failure Modes + Reason Codes
- persona profile commits are deterministic and reason-coded.
- scope/idempotency failures are fail-closed and reason-coded.

## Audit Emission Requirements Per Capability
- `PH1PERSONA_PROFILE_COMMIT_ROW` emits PH1.J with bounded payload keys:
  - `style_profile_ref`
  - `delivery_policy_ref`
  - `preferences_snapshot_ref`
- read capability emits audit only in replay/diagnostic mode.

## Sources
- `crates/selene_storage/src/repo.rs` (`Ph1PersonaRepo`)
- `docs/DB_WIRING/PH1_PERSONA.md`
