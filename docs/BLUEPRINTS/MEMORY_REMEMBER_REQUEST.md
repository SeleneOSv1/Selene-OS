# MEMORY_REMEMBER_REQUEST Blueprint Record

## 1) Blueprint Header
- `process_id`: `MEMORY_REMEMBER_REQUEST`
- `intent_type`: `MEMORY_REMEMBER_REQUEST`
- `version`: `v1`
- `status`: `DRAFT`

## 2) Required Inputs
- `tenant_id`
- `requester_user_id`
- `identity_context` (`speaker_assertion_ok` for voice or signed-in user for text)
- `atom_key`
- `atom_payload`
- `retention_tier` (optional)
- `exposure_policy` (optional)
- `idempotency_key`

## 3) Success Output Schema
```text
atom_event_id: uint64
atom_key: string
atom_state: enum (STORED | UPDATED)
reason_code: string
```

## 4) Ordered Engine Steps

| step_id | engine_name | capability_id | required_fields | produced_fields | side_effects | timeout_ms | max_retries | retry_backoff_ms | retryable_reason_codes |
|---|---|---|---|---|---|---:|---:|---:|---|
| MEMORY_REMEMBER_REQUEST_S01 | PH1.M | MEM_ATOM_UPSERT | tenant_id, requester_user_id, identity_context=VERIFIED, atom_key, atom_payload, provenance=USER_REQUEST, reason_code | atom_event_id, atom_key, atom_state | INTERNAL_DB_WRITE | 350 | 1 | 100 | [MEM_IDEMPOTENCY_REPLAY] |

## 5) Confirmation Points
- none

## 6) Simulation Requirements
- none

## 7) Refusal Conditions
- identity unknown/unverified -> `MEM_POLICY_BLOCKED` (refuse or request reauth)
- missing key/value or ambiguous request -> `MEM_CLARIFY_REQUIRED`

## 8) Acceptance Tests
- `AT-PBS-MEMORY-REMEMBER-01`: Stores atom with provenance and reason_code
- `AT-PBS-MEMORY-REMEMBER-02`: Sensitive exposure obeyed (`SAFE_TO_TEXT` vs `INTERNAL_ONLY`)
- `AT-PBS-MEMORY-REMEMBER-03`: No guessing on missing key/value -> `NEEDS_CLARIFY`
