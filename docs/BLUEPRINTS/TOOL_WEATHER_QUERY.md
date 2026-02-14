# TOOL_WEATHER_QUERY Blueprint Record

## 1) Blueprint Header
- `process_id`: `TOOL_WEATHER_QUERY`
- `intent_type`: `TOOL_WEATHER_QUERY`
- `version`: `v1`
- `status`: `ACTIVE`

## 1A) Contract Boundary
- This blueprint defines orchestration flow only.
- PH1.E writes audit rows only (ToolOk/ToolFail). No domain state is mutated.

## 2) Required Inputs
- `tenant_id`
- `requester_user_id`
- `identity_context` (`speaker_assertion_ok` for voice or signed-in user for text)
- `location` (required)
- `start_date` (optional)
- `duration_days` (optional; default 7)
- `idempotency_key`

## 3) Success Output Schema
```text
tool_name: string (=WEATHER)
weather_result: object
reason_code: string
```

## 4) Ordered Engine Steps
| step_id | engine_name | capability_id | required_fields | produced_fields | side_effects | timeout_ms | max_retries | retry_backoff_ms | retryable_reason_codes |
|---|---|---|---|---|---|---:|---:|---:|---|
| WEATHER_S01 | PH1.E | PH1E_TOOL_OK_COMMIT_ROW | tenant_id, correlation_id, turn_id, user_id, device_id, tool_name=WEATHER, query_hash, cache_status, reason_code, idempotency_key | audit_event_id | DB_WRITE | 250 | 1 | 100 | [E_FAIL_TIMEOUT] |
| WEATHER_S02 | PH1.X | PH1X_RESPOND_COMMIT_ROW | weather_result | response_text | DB_WRITE | 250 | 1 | 100 | [OS_RESPONSE_RETRYABLE] |

Notes:
- The actual weather lookup is performed by the read-only tool router; PH1.E records tool outcome only.
- If tool fails, Selene OS uses PH1E_TOOL_FAIL_COMMIT_ROW instead of TOOL_OK.

## 5) Confirmation Points
- none

## 6) Simulation Requirements
- `TOOL_WEATHER_QUERY_COMMIT`

## 7) Refusal Conditions
- identity unknown/unverified -> `TOOL_POLICY_BLOCKED`
- missing location after clarify budget -> `NLP_CLARIFY_MISSING_FIELD`

## 8) Acceptance Tests
- `AT-PBS-TOOLWEATHER-01`: Weather lookup produces ToolOk audit row and a response.
- `AT-PBS-TOOLWEATHER-02`: Tool failure produces ToolFail audit row and a safe refusal.
