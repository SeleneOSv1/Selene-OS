# LINK_OPEN_ACTIVATE Blueprint Record

## 1) Blueprint Header
- `process_id`: `LINK_OPEN_ACTIVATE`
- `intent_type`: `LINK_OPEN_ACTIVATE`
- `version`: `v1`
- `status`: `ACTIVE`

## 2) Required Inputs
- `token_id`
- `device_fingerprint`
- `idempotency_key`

## 3) Success Output Schema
```text
token_id: string
draft_id: string
activation_status: enum (OPENED | ACTIVATED)
missing_required_fields: string[]
```

## 4) Ordered Engine Steps

| step_id | engine_name | capability_id | required_fields | produced_fields | side_effects | timeout_ms | max_retries | retry_backoff_ms | retryable_reason_codes |
|---|---|---|---|---|---|---:|---:|---:|---|
| LINK_OPEN_S01 | PH1.LINK | PH1LINK_INVITE_OPEN_ACTIVATE_COMMIT_ROW | token_id, device_fingerprint, idempotency_key | activation_status, draft_id, missing_required_fields | DB_WRITE (simulation-gated) | 600 | 2 | 250 | [LINK_OPEN_RETRYABLE] |
| LINK_OPEN_S02 | PH1.X | PH1X_RESPOND_COMMIT_ROW | activation_status | user-facing handoff/refusal response | DB_WRITE | 250 | 1 | 100 | [OS_RESPONSE_RETRYABLE] |

## 5) Confirmation Points
- none (open/activate is deterministic, policy-gated, and simulation-gated).

## 6) Simulation Requirements
- `LINK_INVITE_OPEN_ACTIVATE_COMMIT`
- `LINK_INVITE_FORWARD_BLOCK_COMMIT` (when device mismatch is detected)

## 7) Refusal Conditions
- token invalid/expired/revoked -> `LINK_TOKEN_INVALID`
- forwarded-link device mismatch -> `LINK_FORWARD_BLOCKED`

## 8) Acceptance Tests
- `AT-PBS-LINKOPEN-01`: Open/activate step requires simulation.
- `AT-PBS-LINKOPEN-02`: Device mismatch must fail closed.
- `AT-PBS-LINKOPEN-03`: Handoff includes `draft_id` when activated.
