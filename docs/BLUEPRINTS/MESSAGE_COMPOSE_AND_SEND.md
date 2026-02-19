# MESSAGE_COMPOSE_AND_SEND Blueprint Record

## 1) Blueprint Header
- process_id: MESSAGE_COMPOSE_AND_SEND
- intent_type: MESSAGE_COMPOSE_AND_SEND
- version: v1
- status: ACTIVE

## 1A) Contract Boundary
- This blueprint defines orchestration flow only.
- Engine behavior/schema/capability contracts are canonical in docs/DB_WIRING/*.md and docs/ECM/*.md.

## 2) Purpose
Allow a user to dictate a message/email in any language (including broken/scrambled speech), get a high-quality draft, review it, classify it (Simple/Priority/Private/Confidential/Emergency), choose receipt behavior, and then send via the chosen delivery channel through simulation-gated delivery.

## 3) Required Inputs (eventual, can be gathered via clarify loop)
- tenant_id
- requester_user_id
- recipient (name/id or address token)
- delivery_method (email | sms | whatsapp | wechat)
- subject_topic (email subject or message topic)
- body_content (dictated content)
- classification (SIMPLE | PRIORITY | PRIVATE | CONFIDENTIAL | EMERGENCY)
- receipt_mode (NONE | SENT | READ | REPLY)
- confirm_send (YES/NO)
- idempotency_key

## 4) Success Output Schema
```text
message_id: string
broadcast_id: string
delivery_method: enum (email|sms|whatsapp|wechat)
delivery_status: enum (SENT|FAILED|PENDING)
receipt_mode: enum (NONE|SENT|READ|REPLY)
reason_code: string
```

## 5) Ordered Engine Steps

| step_id | engine_name | capability_id | required_fields | produced_fields | side_effects | timeout_ms | max_retries | retry_backoff_ms | retryable_reason_codes |
|---|---|---|---|---|---|---:|---:|---:|---|
| MSG_S01 | PH1.X | PH1X_CLARIFY_COMMIT_ROW | delivery_method (if missing) | delivery_method | NONE | 300 | 1 | 100 | [X_CLARIFY_TIMEOUT] |
| MSG_S02 | PH1.X | PH1X_CLARIFY_COMMIT_ROW | recipient (if missing) | recipient | NONE | 300 | 1 | 100 | [X_CLARIFY_TIMEOUT] |
| MSG_S03 | PH1.X | PH1X_CLARIFY_COMMIT_ROW | subject_topic (if missing) | subject_topic | NONE | 300 | 1 | 100 | [X_CLARIFY_TIMEOUT] |
| MSG_S04 | PH1.X | PH1X_CLARIFY_COMMIT_ROW | body_content (if missing) | body_content | NONE | 600 | 1 | 100 | [X_CLARIFY_TIMEOUT] |
| MSG_S05 | PH1.X | PH1X_CLARIFY_COMMIT_ROW | classification (if missing) | classification | NONE | 300 | 1 | 100 | [X_CLARIFY_TIMEOUT] |
| MSG_S06 | PH1.X | PH1X_CLARIFY_COMMIT_ROW | receipt_mode (if missing) | receipt_mode | NONE | 300 | 1 | 100 | [X_CLARIFY_TIMEOUT] |
| MSG_S07 | PH1.X | PH1X_CONFIRM_COMMIT_ROW | confirm_send | confirm_send | NONE | 300 | 1 | 100 | [X_CONFIRM_TIMEOUT] |
| MSG_S08 | PH1.ACCESS.001_PH2.ACCESS.002 | ACCESS_GATE_DECIDE_ROW | requester_user_id, tenant_id, requested_action=MESSAGE_SEND | access_decision | NONE | 250 | 1 | 100 | [ACCESS_SCOPE_VIOLATION] |
| MSG_S09 | PH1.BCAST | BCAST_DRAFT_CREATE | recipient, delivery_method, subject_topic, body_content, classification, receipt_mode | broadcast_id | INTERNAL_DB_WRITE | 400 | 1 | 100 | [BCAST_INPUT_SCHEMA_INVALID] |
| MSG_S10 | PH1.BCAST | BCAST_DELIVER_COMMIT | broadcast_id, recipient, delivery_method, simulation_context | delivery_request_ref | EXTERNAL_DELIVERY_REQUEST (simulation-gated) | 500 | 2 | 200 | [BCAST_DELIVERY_PLAN_INVALID] |
| MSG_S11 | PH1.DELIVERY | DELIVERY_SEND | delivery_request_ref, simulation_context, idempotency_key | delivery_proof_ref, delivery_status | EXTERNAL_SEND (simulation-gated) | 700 | 2 | 300 | [DELIVERY_CHANNEL_UNAVAILABLE, DELIVERY_PROVIDER_SEND_FAILED] |
| MSG_S12 | PH1.LEARN | LEARN_SIGNAL_AGGREGATE | tenant_id, feedback_signals(optional) | selected_artifact_id | NONE | 300 | 1 | 100 | [PH1_LEARN_UPSTREAM_INPUT_MISSING] |

Notes:
- Multilingual unraveling is system-wide via PH1.LANG + PH1.SRL + PH1.NLP pipeline before PH1.X decisions.
- Never ask twice is enforced by Selene OS WorkOrder prompt/field dedupe rules.

## 6) Confirmation Points
- MSG_S07 is mandatory final confirmation before any send commit.

## 7) Simulation Requirements
- BCAST_CREATE_DRAFT
- BCAST_DELIVER_COMMIT
- DELIVERY_SEND_COMMIT

## 8) Refusal / Escalation Conditions
- If access_decision = DENY -> refuse with reason-coded access outcome.
- If access_decision = ESCALATE (AP approval required) -> Selene OS runs AP approval flow via PH1.BCAST and waits.
- If delivery_method=sms and sms_app_setup_complete=false -> refuse send commit path (fail closed).

## 9) Acceptance Tests
- AT-PBS-MESSAGE-01: SMS path requires completed SMS setup before send commit.
- AT-PBS-MESSAGE-02: Delivery method + recipient + subject are never re-asked once captured (system-wide no-repeat).
- AT-PBS-MESSAGE-03: Confirmation gate prevents sending without confirm_send=YES.
- AT-PBS-MESSAGE-04: Capability IDs referenced resolve to active ECM entries.
