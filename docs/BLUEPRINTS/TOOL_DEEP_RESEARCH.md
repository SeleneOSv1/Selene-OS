# TOOL_DEEP_RESEARCH Blueprint Record

## 1) Blueprint Header
- `process_id`: `TOOL_DEEP_RESEARCH`
- `intent_type`: `TOOL_DEEP_RESEARCH`
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
tool_name: string (=DEEP_RESEARCH)
research_report: object
reason_code: string
```

## 4) Ordered Engine Steps
| step_id | engine_name | capability_id | required_fields | produced_fields | side_effects | timeout_ms | max_retries | retry_backoff_ms | retryable_reason_codes |
|---|---|---|---|---|---|---:|---:|---:|---|
| DEEP_RESEARCH_S01 | PH1.E | PH1E_TOOL_DEEP_RESEARCH_QUERY | tenant_id, correlation_id, turn_id, user_id, device_id, tool_name=DEEP_RESEARCH, query_hash, cache_status, reason_code | tool_response | READ_ONLY | 250 | 1 | 100 | [E_FAIL_TIMEOUT] |
| DEEP_RESEARCH_S02 | PH1.X | PH1X_TOOL_RESPONSE_RENDER | tool_response | response_text, source_list, retrieved_at | READ_ONLY | 250 | 1 | 100 | [OS_RESPONSE_RETRYABLE] |

Notes:
- Deep research synthesis executes in PH1.E under strict budget and policy context.
- Output must include provenance (`source urls` + `retrieved_at`) from PH1.E response metadata.

## 5) Confirmation Points
- none

## 6) Simulation Requirements
- none (read-only tool lane)

## 7) Refusal Conditions
- policy blocks public web lookup -> `E_FAIL_POLICY_BLOCK`
- query missing/invalid after clarify budget -> `NLP_CLARIFY_MISSING_FIELD`

## 8) Acceptance Tests
- `AT-PBS-TOOLRESEARCH-01`: Deep research routes to PH1.E tool dispatch (not simulation dispatch).
- `AT-PBS-TOOLRESEARCH-02`: Response includes provenance (`source`, `retrieved_at`) and remains read-only.
