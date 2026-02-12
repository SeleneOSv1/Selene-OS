# MESSAGE_COMPOSE_AND_SEND Blueprint Record

## 1) Blueprint Header
- `process_id`: `MESSAGE_COMPOSE_AND_SEND`
- `intent_type`: `MESSAGE_COMPOSE_AND_SEND`
- `version`: `v1`
- `status`: `DRAFT`

## 2) Required Inputs
- `recipient`
- `delivery_method` (`email | sms | whatsapp | wechat`)
- `subject_topic`
- `body_content`
- `classification` (`SIMPLE | PRIVATE | CONFIDENTIAL | PRIORITY | EMERGENCY`)
- `receipt_mode` (`NONE | SENT | READ | REPLY`)
- `confirmation`
- `tenant_id`
- `requester_user_id`
- `idempotency_key`

## 3) Success Output Schema
```text
message_id: string
broadcast_id: string
recipient_delivery_state: enum (QUEUED | SENT | DELIVERED)
receipt_mode: enum (NONE | SENT | READ | REPLY)
reason_code: string
```

## 4) Ordered Engine Steps

| step_id | engine_name | capability_id | required_fields | produced_fields | side_effects | timeout_ms | max_retries | retry_backoff_ms | retryable_reason_codes |
|---|---|---|---|---|---|---:|---:|---:|---|
| MSG_SEND_S01 | PH1.ONBOARDING_SMS | SMS_SETUP_CHECK | delivery_method, tenant_id, requester_user_id | sms_app_setup_complete, setup_state | NONE | 250 | 1 | 100 | [SMS_SETUP_USER_SCOPE_INVALID] |
| MSG_SEND_S02 | PH1.ONBOARDING_SMS | SMS_SETUP_PROMPT | setup_state (when delivery_method=sms and sms_app_setup_complete=false), prompt_dedupe_key | prompt_emitted | NONE | 300 | 1 | 100 | [SMS_SETUP_PROMPT_DEDUPE_SUPPRESSED] |
| MSG_SEND_S03 | PH1.ONBOARDING_SMS | SMS_SETUP_CONFIRM | sms_read_permission_ok, sms_send_permission_ok (when delivery_method=sms and setup incomplete), simulation_context | sms_app_setup_complete | INTERNAL_DB_WRITE (simulation-gated) | 400 | 1 | 100 | [SMS_SETUP_SIMULATION_CONTEXT_MISSING] |
| MSG_SEND_S04 | PH1.X | PH1X_CLARIFY_COMMIT_ROW | delivery_method (when missing) | delivery_method | DB_WRITE | 300 | 1 | 100 | [X_FAIL_PENDING_STATE_INVALID] |
| MSG_SEND_S05 | PH1.X | PH1X_CLARIFY_COMMIT_ROW | subject_topic (when missing) | subject_topic | DB_WRITE | 300 | 1 | 100 | [X_FAIL_PENDING_STATE_INVALID] |
| MSG_SEND_S06 | PH1.LANG | LANG_MULTIPLE_DETECT | body_content | language_segments, dominant_language_tag | NONE | 200 | 1 | 100 | [PH1_LANG_SEGMENTATION_FAILED] |
| MSG_SEND_S07 | PH1.SRL | SRL_FRAME_BUILD | body_content, language_segments | normalized_semantic_frames | NONE | 200 | 1 | 100 | [PH1_SRL_VALIDATION_FAILED] |
| MSG_SEND_S08 | PH1.NLP | PH1NLP_INTENT_DRAFT_COMMIT_ROW | normalized_semantic_frames, recipient, delivery_method, subject_topic | intent_draft | DB_WRITE | 250 | 1 | 100 | [NLP_INPUT_MISSING] |
| MSG_SEND_S09 | PH1.WRITE | PH1WRITE_FORMAT_COMMIT_ROW | intent_draft, body_content, language_segments | formatted_draft | DB_WRITE | 250 | 1 | 100 | [WRITE_FORMAT_INVALID] |
| MSG_SEND_S10 | PH1.X | PH1X_RESPOND_COMMIT_ROW | formatted_draft | reviewed_draft | DB_WRITE | 300 | 1 | 100 | [X_FAIL_PENDING_STATE_INVALID] |
| MSG_SEND_S11 | PH1.X | PH1X_CONFIRM_COMMIT_ROW | classification, receipt_mode, confirmation | confirm_state | DB_WRITE | 300 | 1 | 100 | [X_FAIL_PENDING_STATE_INVALID] |
| MSG_SEND_S12 | PH1.ACCESS.001_PH2.ACCESS.002 | ACCESS_GATE_DECIDE_ROW | requester_user_id, requested_action=MESSAGE_SEND, delivery_method, classification | access_decision | NONE | 250 | 1 | 100 | [ACCESS_SCOPE_VIOLATION, ACCESS_SMS_SETUP_REQUIRED] |
| MSG_SEND_S13 | PH1.BCAST | BCAST_DRAFT_CREATE | recipient, delivery_method, subject_topic, formatted_draft, classification, receipt_mode, simulation_context | broadcast_id | INTERNAL_DB_WRITE | 400 | 1 | 100 | [BCAST_INPUT_SCHEMA_INVALID] |
| MSG_SEND_S14 | PH1.BCAST | BCAST_DELIVER_COMMIT | broadcast_id, recipient, delivery_method, simulation_context | delivery_request_ref, recipient_state | EXTERNAL_DELIVERY_REQUEST (simulation-gated) | 500 | 2 | 200 | [BCAST_DELIVERY_PLAN_INVALID] |
| MSG_SEND_S15 | PH1.DELIVERY | DELIVERY_SEND | delivery_request_ref, recipient, delivery_method, simulation_context | delivery_attempt_id, delivery_status | EXTERNAL_SEND (simulation-gated) | 700 | 2 | 300 | [DELIVERY_CHANNEL_UNAVAILABLE, DELIVERY_PROVIDER_SEND_FAILED] |
| MSG_SEND_S16 | PH1.LEARNING_ADAPTIVE | LEARN_DRAFT_FEEDBACK | draft_ref, user_feedback(optional), simulation_context | quality_delta_bucket | INTERNAL_DB_WRITE (simulation-gated) | 300 | 1 | 100 | [LEARN_SIMULATION_CONTEXT_MISSING] |

## 5) Confirmation Points
- `MSG_SEND_S11` is mandatory final confirmation before any send commit.

## 6) Simulation Requirements
- `SMS_SETUP_SIM`
- `BCAST_CREATE_DRAFT`
- `BCAST_DELIVER_COMMIT`
- `DELIVERY_SEND_COMMIT`
- `LEARN_MODEL_UPDATE_SIM`

## 7) Refusal Conditions
- Access decision `DENY` at `MSG_SEND_S12` -> refuse with access reason code.
- Access decision `ESCALATE` at `MSG_SEND_S12` -> route AP escalation flow via PH1.BCAST and wait.
- SMS path with `sms_app_setup_complete=false` and unresolved setup -> refuse send commit path.

## 8) Acceptance Tests
- `AT-PBS-MESSAGE-01`: SMS path requires completed SMS setup before send commit.
- `AT-PBS-MESSAGE-02`: Mixed-language dictation is segmented and rendered in correct response language.
- `AT-PBS-MESSAGE-03`: Once delivery method and subject are confirmed, prompt dedupe prevents repeated asks.
- `AT-PBS-MESSAGE-04`: Capability IDs in steps resolve to active ECM entries.
