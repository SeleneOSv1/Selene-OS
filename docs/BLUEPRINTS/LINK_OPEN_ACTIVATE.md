# LINK_OPEN_ACTIVATE Blueprint Record

## 1) Blueprint Header
- `process_id`: `LINK_OPEN_ACTIVATE`
- `intent_type`: `LINK_OPEN_ACTIVATE`
- `version`: `v1`
- `status`: `ACTIVE`

## 1A) Contract Boundary
- This blueprint defines orchestration flow only.
- Engine behavior/schema/capability contracts are canonical in `docs/DB_WIRING/*.md` and `docs/ECM/*.md`.

## 1B) Handoff Contract
- `LINK_OPEN_ACTIVATE` output is authoritative for:
  - token validity (expired/revoked/invalid)
  - first-open device_fingerprint binding
  - draft_id resolution
- On success, Selene OS starts ONB_INVITED with:
  - draft_id (required)
  - device_fingerprint (required)
  - token_id (required at activation; optional for ONB trace/audit only)

## 2) Required Inputs
- token_id
- device_fingerprint
- idempotency_key

## 3) Success Output Schema
```text
token_id: string
draft_id: string
activation_status: enum (ACTIVATED | BLOCKED | EXPIRED | REVOKED | CONSUMED)
missing_required_fields: string[]
bound_device_fingerprint_hash: string
```

## 4) Ordered Engine Steps
| step_id | engine_name | capability_id | required_fields | produced_fields | side_effects | timeout_ms | max_retries | retry_backoff_ms | retryable_reason_codes |
|---|---|---|---|---|---|---:|---:|---:|---|
| LINK_OPEN_S01 | PH1.LINK | PH1LINK_INVITE_OPEN_ACTIVATE_COMMIT_ROW | token_id, device_fingerprint, idempotency_key | activation_status, draft_id, missing_required_fields, bound_device_fingerprint_hash | DB_WRITE (simulation-gated) | 600 | 2 | 250 | [LINK_OPEN_RETRYABLE] |
| LINK_OPEN_S02 | PH1.X | PH1X_RESPOND_COMMIT_ROW | activation_status | user-facing handoff/refusal response | DB_WRITE | 250 | 1 | 100 | [OS_RESPONSE_RETRYABLE] |

## 5) Confirmation Points
- none (open/activate is deterministic, policy-gated, and simulation-gated).

## 6) Simulation Requirements
- LINK_INVITE_OPEN_ACTIVATE_COMMIT
- LINK_INVITE_FORWARD_BLOCK_COMMIT (when device mismatch is detected)

## 7) Refusal Conditions
- token invalid/expired/revoked/consumed -> LINK_TOKEN_INVALID
- forwarded-link device mismatch -> LINK_FORWARD_BLOCKED

## 8) Acceptance Tests
- AT-PBS-LINKOPEN-01: Open/activate step requires simulation.
- AT-PBS-LINKOPEN-02: Device mismatch must fail closed.
- AT-PBS-LINKOPEN-03: Handoff includes draft_id when activated.
- AT-PBS-LINKOPEN-04: Successful activation returns draft_id + missing_required_fields and binds token to device_fingerprint.
