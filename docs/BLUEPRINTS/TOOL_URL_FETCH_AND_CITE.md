# TOOL_URL_FETCH_AND_CITE Blueprint Record

## 1) Blueprint Header
- `process_id`: `TOOL_URL_FETCH_AND_CITE`
- `intent_type`: `TOOL_URL_FETCH_AND_CITE`
- `version`: `v1`
- `status`: `ACTIVE`

## 1A) Contract Boundary
- This blueprint defines orchestration flow only.
- Read-only tool lane: no domain state mutation and no simulation commit semantics.

## 2) Required Inputs
- `tenant_id`
- `requester_user_id`
- `identity_context` (`speaker_assertion_ok` for voice or signed-in user for text)
- `url_or_query` (required; may contain full URL)
- `locale` (optional)
- `idempotency_key`

## 3) Success Output Schema
```text
tool_name: string (=URL_FETCH_AND_CITE)
url_fetch_result: object
reason_code: string
```

## 4) Ordered Engine Steps
| step_id | engine_name | capability_id | required_fields | produced_fields | side_effects | timeout_ms | max_retries | retry_backoff_ms | retryable_reason_codes |
|---|---|---|---|---|---|---:|---:|---:|---|
| URL_FETCH_S01 | PH1.E | PH1E_TOOL_URL_FETCH_AND_CITE_QUERY | tenant_id, correlation_id, turn_id, user_id, device_id, tool_name=URL_FETCH_AND_CITE, query_hash, cache_status, reason_code | tool_response | READ_ONLY | 250 | 1 | 100 | [E_FAIL_TIMEOUT] |
| URL_FETCH_S02 | PH1.X | PH1X_TOOL_RESPONSE_RENDER | tool_response | response_text, source_list, retrieved_at, citation_list | READ_ONLY | 250 | 1 | 100 | [OS_RESPONSE_RETRYABLE] |

Notes:
- The fetch and citation extraction executes in PH1.E under strict budget and policy context.
- Output must include provenance (`source urls` + `retrieved_at`) and citation list from PH1.E metadata/result payload.

## 5) Confirmation Points
- none

## 6) Simulation Requirements
- none (read-only tool lane)

## 7) Refusal Conditions
- policy blocks public URL fetch -> `E_FAIL_POLICY_BLOCK`
- missing/invalid URL-or-query after clarify budget -> `NLP_CLARIFY_MISSING_FIELD`

## 8) Acceptance Tests
- `AT-PBS-TOOLURL-01`: URL fetch routes to PH1.E tool dispatch (not simulation dispatch).
- `AT-PBS-TOOLURL-02`: Response includes provenance (`source`, `retrieved_at`) and citations, and remains read-only.
