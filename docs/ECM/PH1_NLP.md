# PH1.NLP ECM Spec

## Engine Header
- `engine_id`: `PH1.NLP`
- `purpose`: Persist deterministic NLP decision outputs (`intent_draft`, `clarify`, `chat`) as bounded audit rows.
- `data_owned`: `audit_events` writes in PH1.NLP scope
- `version`: `v1`
- `status`: `ACTIVE`

## Capability List

### `PH1NLP_INTENT_DRAFT_COMMIT_ROW`
- `name`: Commit NLP intent draft decision
- `input_schema`: `(now, tenant_id, correlation_id, turn_id, session_id?, user_id, device_id, intent_type, overall_confidence, reason_code, idempotency_key)`
- `output_schema`: `Result<AuditEventId, StorageError>`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `DECLARED (DB_WRITE)`

### `PH1NLP_CLARIFY_COMMIT_ROW`
- `name`: Commit NLP clarify decision
- `input_schema`: `(now, tenant_id, correlation_id, turn_id, session_id?, user_id, device_id, what_is_missing, reason_code, idempotency_key)`
- `output_schema`: `Result<AuditEventId, StorageError>`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `DECLARED (DB_WRITE)`

### `PH1NLP_CHAT_COMMIT_ROW`
- `name`: Commit NLP chat/default decision
- `input_schema`: `(now, tenant_id, correlation_id, turn_id, session_id?, user_id, device_id, reason_code, idempotency_key)`
- `output_schema`: `Result<AuditEventId, StorageError>`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `DECLARED (DB_WRITE)`

### `PH1NLP_READ_AUDIT_ROWS`
- `name`: Read PH1.NLP audit rows for one correlation thread
- `input_schema`: `correlation_id`
- `output_schema`: `AuditEvent[]`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `NONE`

## Failure Modes + Reason Codes
- intent/clarify/chat outputs must always carry PH1.NLP deterministic reason codes, including:
  - `NLP_INTENT_OK`
  - `NLP_INTENT_UNKNOWN`
  - `NLP_MULTI_INTENT`
  - `NLP_CLARIFY_MISSING_FIELD`
  - `NLP_CLARIFY_AMBIGUOUS_REFERENCE`
  - `NLP_UNCERTAIN_SPAN`
  - `NLP_CHAT_DEFAULT`
- scope/contract failures are fail-closed with deterministic PH1.NLP reason coding.

## Audit Emission Requirements Per Capability
- write capabilities emit PH1.J audit rows with bounded keys only:
  - `decision`
  - `intent_type`
  - `overall_confidence`
  - `what_is_missing`
- read capability emits audit only in explicit replay/diagnostic mode.

## Sources
- `crates/selene_storage/src/repo.rs` (`Ph1NlpRepo`)
- `docs/DB_WIRING/PH1_NLP.md`
