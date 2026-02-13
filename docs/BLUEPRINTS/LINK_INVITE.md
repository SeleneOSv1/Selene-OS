# LINK_INVITE Blueprint Record

## 1) Blueprint Header
- `process_id`: `LINK_INVITE`
- `intent_type`: `LINK_INVITE`
- `version`: `v1`
- `status`: `ACTIVE`

## 1A) Contract Boundary
- This blueprint defines orchestration flow only.
- Engine behavior/schema/capability contracts are canonical in `docs/DB_WIRING/*.md` and `docs/ECM/*.md`.

## 2) Required Inputs
- `tenant_id` (required for `invitee_type=EMPLOYEE`)
- `inviter_user_id`
- `recipient_contact`
- `delivery_method`
- `invitee_type` (`COMPANY | CUSTOMER | EMPLOYEE | FAMILY_MEMBER | FRIEND | ASSOCIATE`)
- `prefilled_profile_fields` (optional bounded map)
- `idempotency_key`

## 2A) Invitee Type + Prefill Clarify Discipline
- If `invitee_type` is missing or ambiguous, PH1.X asks exactly one clarify question before continuing.
- After minimum fields are captured, Selene may run an optional prefill loop:
  - “Anything else you want me to include so the invitee doesn’t type it?”
  - one question at a time; inviter can stop at any time.

## 3) Success Output Schema
```text
draft_id: string
token_id: string
link_url: string
missing_required_fields: string[]
```

## 4) Ordered Engine Steps

| step_id | engine_name | capability_id | required_fields | produced_fields | side_effects | timeout_ms | max_retries | retry_backoff_ms | retryable_reason_codes |
|---|---|---|---|---|---|---:|---:|---:|---|
| LINK_INVITE_S01 | PH1.C | PH1C_TRANSCRIPT_OK_COMMIT_ROW | correlation_id, turn_id, transcript_hash | transcript_ok evidence row | DB_WRITE | 1200 | 1 | 150 | [STT_FAIL_PROVIDER_TIMEOUT, STT_FAIL_NETWORK_UNAVAILABLE] |
| LINK_INVITE_S02 | PH1.NLP | PH1NLP_INTENT_DRAFT_COMMIT_ROW | transcript_ok, intent_type=LINK_INVITE | intent_draft | DB_WRITE | 200 | 1 | 100 | [NLP_INPUT_MISSING] |
| LINK_INVITE_S03 | PH1.X | PH1X_CONFIRM_COMMIT_ROW | intent_draft | confirmation prompt state | DB_WRITE | 300 | 1 | 100 | [OS_CONFIRM_TIMEOUT] |
| LINK_INVITE_S04 | PH1.ACCESS.001_PH2.ACCESS.002 | ACCESS_GATE_DECIDE_ROW | inviter_user_id, tenant_id, requested_action=LINK_INVITE | access_decision | NONE | 250 | 1 | 100 | [ACCESS_SCOPE_VIOLATION] |
| LINK_INVITE_S05 | PH1.ACCESS.001_PH2.ACCESS.002 | ACCESS_APPLY_OVERRIDE_COMMIT_ROW | access_decision=ESCALATE, approved_by_user_id, simulation_id | override_applied | DB_WRITE (simulation-gated) | 600 | 2 | 250 | [ACCESS_IDEMPOTENCY_REPLAY] |
| LINK_INVITE_S06 | PH1.LINK | PH1LINK_INVITE_GENERATE_DRAFT_ROW | inviter_user_id, invitee_type, recipient_contact, delivery_method, tenant_id, prefilled_profile_fields | draft_id, token_id, link_url, missing_required_fields | DB_WRITE (simulation-gated) | 700 | 2 | 250 | [LINK_GENERATE_RETRYABLE] |
| LINK_INVITE_S07 | PH1.LINK | PH1LINK_INVITE_DRAFT_UPDATE_COMMIT_ROW | draft_id, creator_update_fields, idempotency_key | updated draft + recomputed missing_required_fields | DB_WRITE (simulation-gated) | 700 | 2 | 250 | [LINK_DRAFT_UPDATE_RETRYABLE] |
| LINK_INVITE_S08 | PH1.X | PH1X_RESPOND_COMMIT_ROW | draft_id, token_id, link_url | response prompt: “Link generated. Do you want me to send it now?” | DB_WRITE | 250 | 1 | 100 | [OS_RESPONSE_RETRYABLE] |

S06 persistence note:
- `PH1LINK_INVITE_GENERATE_DRAFT_ROW` stores `invitee_type` and `prefilled_profile_fields` in the `onboarding_draft` payload and computes schema-driven `missing_required_fields`.

## 5) Confirmation Points
- `LINK_INVITE_S03` pre-start confirmation of interpreted invite intent.

## 6) Simulation Requirements
- `LINK_INVITE_GENERATE_DRAFT`
- `LINK_INVITE_DRAFT_UPDATE_COMMIT`
- `ACCESS_OVERRIDE_TEMP_GRANT_COMMIT` (conditional escalation path)
- `ACCESS_OVERRIDE_PERM_GRANT_COMMIT` (conditional escalation path)

Delivery note:
- Delivery is executed by `LINK_DELIVER_INVITE` (`PH1.BCAST` + `PH1.DELIVERY`). `PH1.LINK` does not send.

## 7) Refusal Conditions
- Access denied at `LINK_INVITE_S04` -> `ACCESS_SCOPE_VIOLATION`
- Missing required invite fields after clarify budget -> `NLP_CLARIFY_MISSING_FIELD`
- User declines confirmation at `LINK_INVITE_S03` -> `USER_DECLINED_CONFIRMATION`

## 8) Acceptance Tests
- `AT-PBS-LINK-01`: No blueprint match -> no process start.
- `AT-PBS-LINK-02`: Mandatory confirmation before link generation commit path.
- `AT-PBS-LINK-03`: Every side-effect step references a simulation.
- `AT-PBS-LINK-04`: Capability IDs resolve to active ECM entries.
- `AT-PBS-LINK-05`: `LINK_INVITE` generates `link_url` but does not deliver; delivery is a separate blueprint.
