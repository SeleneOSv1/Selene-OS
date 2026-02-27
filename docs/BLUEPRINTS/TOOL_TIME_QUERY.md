# TOOL_TIME_QUERY Blueprint Record

## 1) Blueprint Header
- `process_id`: `TOOL_TIME_QUERY`
- `intent_type`: `TOOL_TIME_QUERY`
- `version`: `v1`
- `status`: `ACTIVE`

## 1A) Contract Boundary
- This blueprint defines orchestration flow only.
- PH1.E writes audit rows only (ToolOk/ToolFail). No domain state is mutated.

## 2) Required Inputs
- `tenant_id`
- `requester_user_id`
- `identity_context` (`speaker_assertion_ok` for voice or signed-in user for text)
- `timezone_hint` (optional; default uses user/session timezone)
- `idempotency_key`

## 3) Success Output Schema
```text
tool_name: string (=TIME)
time_result: object
reason_code: string
```

## 4) Ordered Engine Steps
| step_id | engine_name | capability_id | required_fields | produced_fields | side_effects | timeout_ms | max_retries | retry_backoff_ms | retryable_reason_codes |
|---|---|---|---|---|---|---:|---:|---:|---|
| TIME_S01 | PH1.E | PH1E_TOOL_TIME_QUERY | tenant_id, correlation_id, turn_id, user_id, device_id, tool_name=TIME, query_hash, cache_status, reason_code | tool_response | READ_ONLY | 250 | 1 | 100 | [E_FAIL_TIMEOUT] |
| TIME_S02 | PH1.X | PH1X_TOOL_RESPONSE_RENDER | tool_response | response_text | READ_ONLY | 250 | 1 | 100 | [OS_RESPONSE_RETRYABLE] |

Notes:
- The actual time lookup is performed by the read-only tool router.
- Any audit append occurs in the standard PH1.WRITE/audit lane, not a side-effect simulation for this blueprint.

## 5) Confirmation Points
- none

## 6) Simulation Requirements
- none (read-only tool lane)

## 7) Refusal Conditions
- identity unknown/unverified -> `TOOL_POLICY_BLOCKED`

## 8) Acceptance Tests
- `AT-PBS-TOOLTIME-01`: Time lookup produces ToolOk audit row and a response.
- `AT-PBS-TOOLTIME-02`: Tool failure produces ToolFail audit row and a safe refusal.
