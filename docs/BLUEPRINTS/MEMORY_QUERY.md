# MEMORY_QUERY Blueprint Record

## 1) Blueprint Header
- `process_id`: `MEMORY_QUERY`
- `intent_type`: `MEMORY_QUERY`
- `version`: `v1`
- `status`: `ACTIVE`

## 2) Required Inputs
- `tenant_id`
- `requester_user_id`
- `identity_context` (`speaker_assertion_ok` for voice or signed-in user for text)
- `idempotency_key`

## 3) Success Output Schema
```text
memory_summary_items: object[]
summary_item_count: uint32
summary_scope: enum (SAFE_TO_SPEAK | SAFE_TO_TEXT)
reason_code: string
```

## 4) Ordered Engine Steps

| step_id | engine_name | capability_id | required_fields | produced_fields | side_effects | timeout_ms | max_retries | retry_backoff_ms | retryable_reason_codes |
|---|---|---|---|---|---|---:|---:|---:|---|
| MEMORY_QUERY_S01 | PH1.M | MEM_QUERY_SAFE_SUMMARY | tenant_id, requester_user_id, identity_context=VERIFIED | memory_summary_items, summary_item_count, summary_scope | NONE | 250 | 1 | 100 | [MEM_POLICY_BLOCKED] |

## 5) Confirmation Points
- none

## 6) Simulation Requirements
- none

## 7) Refusal Conditions
- identity unknown/unverified -> `MEM_POLICY_BLOCKED` (refuse or request reauth)

## 8) Acceptance Tests
- `AT-PBS-MEMORY-QUERY-01`: Verified identity returns bounded safe items only
- `AT-PBS-MEMORY-QUERY-02`: Unknown identity returns refusal / reauth request
