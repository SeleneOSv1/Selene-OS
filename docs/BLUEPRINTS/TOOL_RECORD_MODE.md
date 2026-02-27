# TOOL_RECORD_MODE Blueprint Record

## 1) Blueprint Header
- `process_id`: `TOOL_RECORD_MODE`
- `intent_type`: `TOOL_RECORD_MODE`
- `version`: `v1`
- `status`: `ACTIVE`

## 1A) Contract Boundary
- This blueprint defines orchestration flow only.
- Read-only tool lane: no domain state mutation and no simulation commit semantics.

## 2) Required Inputs
- `tenant_id`
- `requester_user_id`
- `identity_context` (`speaker_assertion_ok` for voice or signed-in user for text)
- `query` (required)
- `locale` (optional)
- `idempotency_key`

## 3) Success Output Schema
```text
tool_name: string (=RECORD_MODE)
record_summary: object
reason_code: string
```

## 4) Ordered Engine Steps
| step_id | engine_name | capability_id | required_fields | produced_fields | side_effects | timeout_ms | max_retries | retry_backoff_ms | retryable_reason_codes |
|---|---|---|---|---|---|---:|---:|---:|---|
| RECORD_MODE_S01 | PH1.E | PH1E_TOOL_RECORD_MODE_QUERY | tenant_id, correlation_id, turn_id, user_id, device_id, tool_name=RECORD_MODE, query_hash, cache_status, reason_code | tool_response | READ_ONLY | 250 | 1 | 100 | [E_FAIL_TIMEOUT] |
| RECORD_MODE_S02 | PH1.X | PH1X_TOOL_RESPONSE_RENDER | tool_response | response_text, source_list, retrieved_at | READ_ONLY | 250 | 1 | 100 | [OS_RESPONSE_RETRYABLE] |

Notes:
- Record mode emits meeting summary + action items + evidence references keyed by transcript chunk and timecode.
- Provenance sources must use recording evidence references (for example `recording://session/...`) instead of web URLs.

## 5) Confirmation Points
- none

## 6) Simulation Requirements
- none (read-only tool lane)

## 7) Refusal Conditions
- query missing/invalid after clarify budget -> `NLP_CLARIFY_MISSING_FIELD`
- tool timeout -> `E_FAIL_TIMEOUT`

## 8) Acceptance Tests
- `AT-PBS-TOOLRECORD-01`: Record mode routes to PH1.E tool dispatch (not simulation dispatch).
- `AT-PBS-TOOLRECORD-02`: Response includes action items + recording evidence refs + provenance metadata.
