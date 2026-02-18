# PH1.X ECM Spec

## Engine Header
- `engine_id`: `PH1.X`
- `purpose`: Persist deterministic PH1.X conversational directives (`confirm`, `clarify`, `respond`, `dispatch`, `wait`) as bounded audit rows.
- `data_owned`: `audit_events` writes in PH1.X scope
- `version`: `v1`
- `status`: `ACTIVE`

## Capability List

### `PH1X_CONFIRM_COMMIT_ROW`
- `name`: Commit confirm directive
- `input_schema`: `(now, tenant_id, correlation_id, turn_id, session_id?, user_id, device_id, work_order_id, work_order_status_snapshot, pending_state, confirm_kind, reason_code, idempotency_key)`
- `output_schema`: `Result<AuditEventId, StorageError>`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `DECLARED (DB_WRITE)`

### `PH1X_CLARIFY_COMMIT_ROW`
- `name`: Commit clarify directive
- `input_schema`: `(now, tenant_id, correlation_id, turn_id, session_id?, user_id, device_id, work_order_id, work_order_status_snapshot, pending_state, what_is_missing, clarification_unit_id, reason_code, idempotency_key)`
- `output_schema`: `Result<AuditEventId, StorageError>`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `DECLARED (DB_WRITE)`

### `PH1X_RESPOND_COMMIT_ROW`
- `name`: Commit respond directive
- `input_schema`: `(now, tenant_id, correlation_id, turn_id, session_id?, user_id, device_id, work_order_id, work_order_status_snapshot, pending_state, response_kind, reason_code, idempotency_key)`
- `output_schema`: `Result<AuditEventId, StorageError>`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `DECLARED (DB_WRITE)`

### `PH1X_DISPATCH_COMMIT_ROW`
- `name`: Commit dispatch directive
- `input_schema`: `(now, tenant_id, correlation_id, turn_id, session_id?, user_id, device_id, work_order_id, work_order_status_snapshot, pending_state, dispatch_target, lease_token_hash?, reason_code, idempotency_key)`
- `output_schema`: `Result<AuditEventId, StorageError>`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `DECLARED (DB_WRITE)`

### `PH1X_WAIT_COMMIT_ROW`
- `name`: Commit wait directive
- `input_schema`: `(now, tenant_id, correlation_id, turn_id, session_id?, user_id, device_id, work_order_id, work_order_status_snapshot, pending_state, wait_kind, reason_code, idempotency_key)`
- `output_schema`: `Result<AuditEventId, StorageError>`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `DECLARED (DB_WRITE)`

### `PH1X_READ_AUDIT_ROWS`
- `name`: Read PH1.X audit rows for one correlation thread
- `input_schema`: `correlation_id`
- `output_schema`: `AuditEvent[]`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `NONE`

## Failure Modes + Reason Codes
- PH1.X outputs must always carry deterministic PH1.X reason_code values from the PH1.X contract path.
- storage scope/idempotency failures are fail-closed and reason-coded in PH1.X audit emissions.
- gating failures are deterministic and reason-coded (`X_FAIL_WORK_ORDER_SCOPE_INVALID`, `X_FAIL_LEASE_REQUIRED`, `X_FAIL_PENDING_STATE_INVALID`).
- continuity failures are deterministic and reason-coded:
  - `X_CONTINUITY_SPEAKER_MISMATCH`: active speaker mismatch against thread continuity; PH1.X returns one clarify and blocks dispatch.
  - `X_CONTINUITY_SUBJECT_MISMATCH`: subject drift while pending exists; PH1.X returns one clarify and blocks dispatch.

## Audit Emission Requirements Per Capability
- write capabilities emit PH1.J rows using:
  - `XConfirm` for confirm
  - `XDispatch` for dispatch
  - `Other` for clarify/respond/wait (with bounded directive payload keys)
- required bounded payload keys include:
  - `directive`
  - `confirm_kind`
  - `what_is_missing`
  - `clarification_unit_id`
  - `response_kind`
  - `dispatch_target`
  - `wait_kind`
  - `work_order_id`
  - `work_order_status_snapshot`
  - `pending_state`
  - `lease_token_hash`
- read capability emits audit only in explicit replay/diagnostic mode.

## Sources
- `crates/selene_storage/src/repo.rs` (`Ph1xConversationRepo`)
- `docs/DB_WIRING/PH1_X.md`

## Related Engine Boundary (`PH1.PRUNE`)
- PH1.X clarify packet construction may consume PH1.PRUNE `selected_missing_field` when the turn has multiple missing fields.
- PH1.X must not treat PH1.PRUNE as authoritative; PH1.X remains responsible for final move selection and fail-closed behavior.

## Related Engine Boundary (`PH1.EXPLAIN`)
- PH1.X can invoke PH1.EXPLAIN only for explicit explain triggers or accountability responses.
- PH1.X must treat PH1.EXPLAIN as advisory output only; no authority or execution semantics are inferred from explanation text.

## Related Engine Boundary (`PH1.EMO.GUIDE`)
- PH1.X may consume PH1.EMO.GUIDE style-profile hints only when `EMO_GUIDE_PROFILE_VALIDATE` returns `validation_status=OK`.
- PH1.X must treat PH1.EMO.GUIDE as advisory tone policy only; no authority, execution, truth, or confirmation semantics can be inferred from EMO.GUIDE output.

## Related Engine Boundary (`PH1.PERSONA`)
- PH1.X may consume PH1.PERSONA style/delivery profile hints only when `PERSONA_PROFILE_VALIDATE` returns `validation_status=OK`.
- PH1.X must treat PH1.PERSONA as advisory tone/delivery policy only; no authority, execution, truth, or confirmation semantics can be inferred from PH1.PERSONA output.
