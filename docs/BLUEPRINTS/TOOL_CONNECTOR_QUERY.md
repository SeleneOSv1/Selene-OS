# TOOL_CONNECTOR_QUERY Blueprint Record

## 1) Blueprint Header
- `process_id`: `TOOL_CONNECTOR_QUERY`
- `intent_type`: `TOOL_CONNECTOR_QUERY`
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
tool_name: string (=CONNECTOR_QUERY)
connector_summary: object
reason_code: string
```

## 4) Ordered Engine Steps
| step_id | engine_name | capability_id | required_fields | produced_fields | side_effects | timeout_ms | max_retries | retry_backoff_ms | retryable_reason_codes |
|---|---|---|---|---|---|---:|---:|---:|---|
| CONNECTOR_QUERY_S01 | PH1.E | PH1E_TOOL_CONNECTOR_QUERY | tenant_id, correlation_id, turn_id, user_id, device_id, tool_name=CONNECTOR_QUERY, query_hash, cache_status, reason_code | tool_response | READ_ONLY | 250 | 1 | 100 | [E_FAIL_TIMEOUT] |
| CONNECTOR_QUERY_S02 | PH1.X | PH1X_TOOL_RESPONSE_RENDER | tool_response | response_text, source_list, retrieved_at | READ_ONLY | 250 | 1 | 100 | [OS_RESPONSE_RETRYABLE] |

Notes:
- Connector query reads user-connected app data (for example: mail/calendar/docs) through PH1.E read-only tooling.
- PH1.E resolves provider scope from query text (`gmail/outlook/calendar/drive/dropbox/slack/notion/onedrive`), defaults to `gmail,calendar,drive` when scope is absent, and returns scope-aware citations/sources.
- Output must include provenance (`source urls` + `retrieved_at`) from PH1.E response metadata.

## 5) Confirmation Points
- none

## 6) Simulation Requirements
- none (read-only tool lane)

## 7) Refusal Conditions
- query missing/invalid after clarify budget -> `NLP_CLARIFY_MISSING_FIELD`
- policy blocks connector query in strict tenant policy mode -> `E_FAIL_POLICY_BLOCK`
- unsupported connector provider requested (for example `salesforce`, `servicenow`, `zendesk`, `hubspot`) -> `E_FAIL_POLICY_BLOCK`

## 8) Acceptance Tests
- `AT-PBS-TOOLCONNECTOR-01`: Connector query routes to PH1.E tool dispatch (not simulation dispatch).
- `AT-PBS-TOOLCONNECTOR-02`: Response includes extracted fields + citations + provenance metadata.
- `AT-PBS-TOOLCONNECTOR-03`: Explicit provider scope limits citations/sources to requested connectors.
- `AT-PBS-TOOLCONNECTOR-04`: Unsupported connector scope fails closed with `E_FAIL_POLICY_BLOCK`.
