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

### `PH1E_TOOL_TIME_QUERY`
- `name`: Execute read-only time lookup query
- `input_schema`: `(tenant_id, correlation_id, turn_id, user_id, device_id, locale?, query, idempotency_key)`
- `output_schema`: `tool_response { tool_name, answer_text, provenance { source, retrieved_at } }`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `NONE`

### `PH1E_TOOL_WEATHER_QUERY`
- `name`: Execute read-only weather lookup query
- `input_schema`: `(tenant_id, correlation_id, turn_id, user_id, device_id, locale?, query, idempotency_key)`
- `output_schema`: `tool_response { tool_name, answer_text, provenance { source, retrieved_at } }`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `NONE`

### `PH1E_TOOL_WEB_SEARCH_QUERY`
- `name`: Execute read-only web search query
- `input_schema`: `(tenant_id, correlation_id, turn_id, user_id, device_id, locale?, query, idempotency_key)`
- `output_schema`: `tool_response { tool_name, answer_text, provenance { source, retrieved_at } }`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `NONE`

### `PH1E_TOOL_NEWS_QUERY`
- `name`: Execute read-only news lookup query
- `input_schema`: `(tenant_id, correlation_id, turn_id, user_id, device_id, locale?, query, idempotency_key)`
- `output_schema`: `tool_response { tool_name, answer_text, provenance { source, retrieved_at } }`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `NONE`

### `PH1E_TOOL_URL_FETCH_AND_CITE_QUERY`
- `name`: Execute read-only URL fetch and citation query
- `input_schema`: `(tenant_id, correlation_id, turn_id, user_id, device_id, url, query?, idempotency_key)`
- `output_schema`: `tool_response { tool_name, answer_text, citations[], provenance { source, retrieved_at } }`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `NONE`

### `PH1E_TOOL_DOCUMENT_UNDERSTAND_QUERY`
- `name`: Execute read-only document extraction/understand query
- `input_schema`: `(tenant_id, correlation_id, turn_id, user_id, device_id, document_ref, query, idempotency_key)`
- `output_schema`: `tool_response { tool_name, answer_text, structured_fields, citations[], provenance { source, retrieved_at } }`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `NONE`

### `PH1E_TOOL_PHOTO_UNDERSTAND_QUERY`
- `name`: Execute read-only image/photo understanding query
- `input_schema`: `(tenant_id, correlation_id, turn_id, user_id, device_id, image_ref, query, idempotency_key)`
- `output_schema`: `tool_response { tool_name, answer_text, structured_fields, citations[], provenance { source, retrieved_at } }`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `NONE`

### `PH1E_TOOL_DATA_ANALYSIS_QUERY`
- `name`: Execute read-only uploaded-data analysis query
- `input_schema`: `(tenant_id, correlation_id, turn_id, user_id, device_id, data_ref, query, idempotency_key)`
- `output_schema`: `tool_response { tool_name, answer_text, structured_fields, citations[], provenance { source, retrieved_at } }`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `NONE`

### `PH1E_TOOL_DEEP_RESEARCH_QUERY`
- `name`: Execute read-only deep-research synthesis query
- `input_schema`: `(tenant_id, correlation_id, turn_id, user_id, device_id, query, locale?, idempotency_key)`
- `output_schema`: `tool_response { tool_name, answer_text, structured_fields, citations[], provenance { source, retrieved_at } }`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `NONE`

### `PH1E_TOOL_RECORD_MODE_QUERY`
- `name`: Execute read-only recording transcript summary/action-item query
- `input_schema`: `(tenant_id, correlation_id, turn_id, user_id, device_id, recording_ref, query, idempotency_key)`
- `output_schema`: `tool_response { tool_name, summary, action_items, evidence_refs, provenance { source, retrieved_at } }`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `NONE`

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
