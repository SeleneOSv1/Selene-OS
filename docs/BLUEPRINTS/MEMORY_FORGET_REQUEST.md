# MEMORY_FORGET_REQUEST Blueprint Record

## 1) Blueprint Header
- `process_id`: `MEMORY_FORGET_REQUEST`
- `intent_type`: `MEMORY_FORGET_REQUEST`
- `version`: `v1`
- `status`: `DRAFT`

## 2) Required Inputs
- `tenant_id`
- `requester_user_id`
- `identity_context` (`speaker_assertion_ok` for voice or signed-in user for text)
- `forget_scope` (required for forget requests)
- `rule_kind` + `rule_key` + `scope` (required for suppression requests)
- `idempotency_key`

## 3) Success Output Schema
```text
forget_result: object
suppression_rule_state: object (optional)
reason_code: string
```

## 4) Ordered Engine Steps

| step_id | engine_name | capability_id | required_fields | produced_fields | side_effects | timeout_ms | max_retries | retry_backoff_ms | retryable_reason_codes |
|---|---|---|---|---|---|---:|---:|---:|---|
| MEMORY_FORGET_REQUEST_S01 | PH1.M | MEM_FORGET | tenant_id, requester_user_id, identity_context=VERIFIED, forget_scope | forget_result | INTERNAL_DB_WRITE | 350 | 1 | 100 | [MEM_IDEMPOTENCY_REPLAY] |
| MEMORY_FORGET_REQUEST_S02 | PH1.M | MEM_SUPPRESSION_SET | tenant_id, requester_user_id, identity_context=VERIFIED, rule_kind, rule_key, scope (when suppression requested) | suppression_rule_state | INTERNAL_DB_WRITE | 350 | 1 | 100 | [MEM_IDEMPOTENCY_REPLAY] |

## 5) Confirmation Points
- none

## 6) Simulation Requirements
- none

## 7) Refusal Conditions
- identity unknown/unverified -> `MEM_POLICY_BLOCKED` (refuse or request reauth)
- invalid forget/suppression request -> `MEM_INVALID_RULE_REQUEST`

## 8) Acceptance Tests
- `AT-PBS-MEMORY-FORGET-01`: Forget deactivates atom/thread and writes ledger event
- `AT-PBS-MEMORY-FORGET-02`: Forget does not rewrite audit/immutable truth
- `AT-PBS-MEMORY-FORGET-03`: Suppression prevents recall (`DO_NOT_MENTION` hit)
