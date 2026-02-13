# LINK_DELIVER_INVITE Blueprint Record

## 1) Blueprint Header
- `process_id`: `LINK_DELIVER_INVITE`
- `intent_type`: `LINK_DELIVER_INVITE`
- `version`: `v1`
- `status`: `DRAFT`

## 1A) Contract Boundary
- This blueprint defines orchestration flow only.
- Engine behavior/schema/capability contracts are canonical in `docs/DB_WIRING/*.md` and `docs/ECM/*.md`.

## 2) Required Inputs
- `tenant_id` (if applicable)
- `requester_user_id`
- `recipient_contact`
- `delivery_method` (`SELENE_APP | SMS | WHATSAPP | WECHAT | EMAIL`)
- `link_url`
- `classification` (`SIMPLE` default; `PRIVATE | CONFIDENTIAL` when policy allows)
- `idempotency_key`

## 3) Success Output Schema
```text
delivery_status: enum (SENT | FAILED)
delivery_proof_ref: string
reason_code: string
```

## 4) Ordered Engine Steps

| step_id | engine_name | capability_id | required_fields | produced_fields | side_effects | timeout_ms | max_retries | retry_backoff_ms | retryable_reason_codes |
|---|---|---|---|---|---|---:|---:|---:|---|
| LINK_DELIVER_S01 | PH1.X | PH1X_CONFIRM_COMMIT_ROW | requester_user_id, recipient_contact, delivery_method, link_url, classification | send_confirmation_state | DB_WRITE | 300 | 1 | 100 | [OS_CONFIRM_TIMEOUT] |
| LINK_DELIVER_S02 | PH1.ACCESS.001_PH2.ACCESS.002 | ACCESS_GATE_DECIDE_ROW | requester_user_id, tenant_id, requested_action=LINK_DELIVER_INVITE, classification | access_decision | NONE | 250 | 1 | 100 | [ACCESS_SCOPE_VIOLATION] |
| LINK_DELIVER_S03 | PH1.BCAST | BCAST_DRAFT_CREATE | requester_user_id, recipient_contact, delivery_method, classification, payload={link_url} | broadcast_id | INTERNAL_DB_WRITE (simulation-gated) | 400 | 1 | 100 | [BCAST_INPUT_SCHEMA_INVALID] |
| LINK_DELIVER_S04 | PH1.BCAST | BCAST_DELIVER_COMMIT | broadcast_id, recipient_contact, delivery_method, simulation_context | delivery_request_ref | EXTERNAL_DELIVERY_REQUEST (simulation-gated) | 500 | 2 | 200 | [BCAST_DELIVERY_PLAN_INVALID] |
| LINK_DELIVER_S05 | PH1.DELIVERY | DELIVERY_SEND | delivery_request_ref, simulation_context, idempotency_key | delivery_proof_ref, delivery_status | EXTERNAL_SEND (simulation-gated) | 700 | 2 | 300 | [DELIVERY_CHANNEL_UNAVAILABLE, DELIVERY_PROVIDER_SEND_FAILED] |
| LINK_DELIVER_S06 | PH1.X | PH1X_RESPOND_COMMIT_ROW | delivery_status, delivery_proof_ref | sender-facing sent/failure response | DB_WRITE | 250 | 1 | 100 | [OS_RESPONSE_RETRYABLE] |

## 5) Confirmation Points
- `LINK_DELIVER_S01` confirms destination, channel, and send intent before delivery commit.

## 6) Simulation Requirements
- `BCAST_CREATE_DRAFT`
- `BCAST_DELIVER_COMMIT`
- `DELIVERY_SEND_COMMIT`
- `ACCESS_OVERRIDE_TEMP_GRANT_COMMIT` (conditional escalation path)
- `ACCESS_OVERRIDE_PERM_GRANT_COMMIT` (conditional escalation path)

## 7) Refusal Conditions
- Access denied at `LINK_DELIVER_S02` -> `ACCESS_SCOPE_VIOLATION`
- User declines send confirmation at `LINK_DELIVER_S01` -> `USER_DECLINED_CONFIRMATION`

## 8) Acceptance Tests
- `AT-PBS-LINKDEL-01`: No delivery without Access+Simulation.
- `AT-PBS-LINKDEL-02`: Delivery is idempotent (no duplicate sends on retry).
- `AT-PBS-LINKDEL-03`: Proof is recorded (`delivery_proof_ref`) and returned.
