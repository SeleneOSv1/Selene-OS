# TOOL_DOCUMENT_UNDERSTAND Blueprint Record

## 1) Blueprint Header
- `process_id`: `TOOL_DOCUMENT_UNDERSTAND`
- `intent_type`: `TOOL_DOCUMENT_UNDERSTAND`
- `version`: `v1`
- `status`: `ACTIVE`

## 1A) Contract Boundary
- This blueprint defines orchestration flow only.
- Read-only tool lane: no domain state mutation and no simulation commit semantics.

## 2) Required Inputs
- `tenant_id`
- `requester_user_id`
- `identity_context` (`speaker_assertion_ok` for voice or signed-in user for text)
- `document_ref_or_query` (required; document payload reference or deterministic query text)
- `locale` (optional)
- `idempotency_key`

## 3) Success Output Schema
```text
tool_name: string (=DOCUMENT_UNDERSTAND)
document_understand_result: object
reason_code: string
```

## 4) Ordered Engine Steps
| step_id | engine_name | capability_id | required_fields | produced_fields | side_effects | timeout_ms | max_retries | retry_backoff_ms | retryable_reason_codes |
|---|---|---|---|---|---|---:|---:|---:|---|
| DOC_UNDERSTAND_S01 | PH1.E | PH1E_TOOL_DOCUMENT_UNDERSTAND_QUERY | tenant_id, correlation_id, turn_id, user_id, device_id, tool_name=DOCUMENT_UNDERSTAND, query_hash, cache_status, reason_code | tool_response | READ_ONLY | 300 | 1 | 100 | [E_FAIL_TIMEOUT] |
| DOC_UNDERSTAND_S02 | PH1.X | PH1X_TOOL_RESPONSE_RENDER | tool_response | response_text, source_list, retrieved_at, extracted_fields, citation_list | READ_ONLY | 300 | 1 | 100 | [OS_RESPONSE_RETRYABLE] |

Notes:
- PH1.E returns structured extraction fields plus citation snippets.
- Output must include provenance (`source urls` + `retrieved_at`) and citations from PH1.E metadata/result payload.

## 5) Confirmation Points
- none

## 6) Simulation Requirements
- none (read-only tool lane)

## 7) Refusal Conditions
- document payload missing or unusable after clarify budget -> `NLP_CLARIFY_MISSING_FIELD`
- extraction validation fails -> `PH1_DOC_VALIDATION_FAILED` or `E_FAIL_INTERNAL_PIPELINE_ERROR`

## 8) Acceptance Tests
- `AT-PBS-TOOLDOC-01`: Document-understand intent routes to PH1.E tool dispatch (not simulation dispatch).
- `AT-PBS-TOOLDOC-02`: Response includes structured extraction fields + citations + provenance and remains read-only.
