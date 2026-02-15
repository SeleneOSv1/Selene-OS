# ONB_REQUIREMENT_BACKFILL Blueprint Record

## 1) Blueprint Header
- `process_id`: `ONB_REQUIREMENT_BACKFILL`
- `intent_type`: `ONB_REQUIREMENT_BACKFILL`
- `version`: `v1`
- `status`: `ACTIVE`

## 1A) Contract Boundary
- This blueprint defines orchestration flow only.
- Engine behavior/schema/capability contracts are canonical in `docs/DB_WIRING/*.md` and `docs/ECM/*.md`.
- Launch policy: this process is entered only from explicit `CurrentAndNew` schema activation scope in `ONB_SCHEMA_MANAGE`.

## 2) Required Inputs
- `tenant_id`
- `actor_user_id`
- `company_id`
- `position_id`
- `schema_version_id`
- `rollout_scope` (`CurrentAndNew` only; `NewHiresOnly` must not enter this blueprint)
- `idempotency_key`
- `activation_handoff_ref` (deterministic link to `ONB_SCHEMA_MANAGE` activation output)

## 3) Success Output Schema
```text
campaign_id: string
position_id: string
state: enum (DRAFT_CREATED | RUNNING | COMPLETED | CANCELED)
pending_target_count: integer
completed_target_count: integer
total_target_count: integer
```

## 4) Ordered Engine Steps

| step_id | engine_name | capability_id | required_fields | produced_fields | side_effects | timeout_ms | max_retries | retry_backoff_ms | retryable_reason_codes |
|---|---|---|---|---|---|---:|---:|---:|---|
| ONB_BACKFILL_S01 | PH1.C | PH1C_TRANSCRIPT_OK_COMMIT_ROW | correlation_id, turn_id, transcript_hash | transcript_ok evidence row | DB_WRITE | 1200 | 1 | 150 | [STT_FAIL_PROVIDER_TIMEOUT, STT_FAIL_NETWORK_UNAVAILABLE] |
| ONB_BACKFILL_S02 | PH1.NLP | PH1NLP_INTENT_DRAFT_COMMIT_ROW | transcript_ok, intent_type=ONB_REQUIREMENT_BACKFILL | intent_draft | DB_WRITE | 200 | 1 | 100 | [NLP_INPUT_MISSING] |
| ONB_BACKFILL_S03 | PH1.X | PH1X_CONFIRM_COMMIT_ROW | company_id, position_id, schema_version_id, rollout_scope=CurrentAndNew | confirmation prompt state | DB_WRITE | 300 | 1 | 100 | [OS_CONFIRM_TIMEOUT] |
| ONB_BACKFILL_S04 | PH1.ACCESS.001_PH2.ACCESS.002 | ACCESS_GATE_DECIDE_ROW | actor_user_id, tenant_id, requested_action=ONB_REQUIREMENT_BACKFILL | access_decision | NONE | 250 | 1 | 100 | [ACCESS_SCOPE_VIOLATION] |
| ONB_BACKFILL_S05 | PH1.ONB | PH1ONB_BACKFILL_START_DRAFT_ROW | actor_user_id, tenant_id, company_id, position_id, schema_version_id, rollout_scope, idempotency_key | campaign_id, state, pending_target_count | DB_WRITE (simulation-gated) | 700 | 2 | 250 | [ONB_BACKFILL_START_RETRYABLE] |
| ONB_BACKFILL_S06 | PH1.BCAST | BCAST_DRAFT_CREATE | tenant_id, sender_user_id=actor_user_id, audience_spec(target_population), classification, content_payload_ref, idempotency_key | broadcast_id, status=draft_created | INTERNAL_DB_WRITE (simulation-gated) | 400 | 1 | 100 | [BCAST_INPUT_SCHEMA_INVALID] |
| ONB_BACKFILL_S07 | PH1.BCAST | BCAST_DELIVER_COMMIT | broadcast_id, recipient_id, delivery_plan_ref, simulation_context, idempotency_key | delivery_request_ref, recipient_state | EXTERNAL_DELIVERY_REQUEST (simulation-gated) | 700 | 2 | 250 | [BCAST_SIMULATION_CONTEXT_MISSING, BCAST_DELIVERY_PLAN_INVALID] |
| ONB_BACKFILL_S08 | PH1.REM | PH1REM_SCHEDULE_COMMIT_ROW | tenant_id, user_id, reminder_type=BCAST_MHP_FOLLOWUP, desired_time, idempotency_key | reminder_id, occurrence_id, scheduled_time | DB_WRITE (simulation-gated) | 700 | 2 | 250 | [REM_FAIL_TIMEZONE_INVALID, REM_FAIL_IDEMPOTENCY_REPLAY] |
| ONB_BACKFILL_S09 | PH1.ONB | PH1ONB_BACKFILL_NOTIFY_COMMIT_ROW | campaign_id, tenant_id, recipient_user_id, idempotency_key | campaign_id, recipient_user_id, target_status | DB_WRITE (simulation-gated; per-recipient progress commit after BCAST/REM handoff) | 700 | 2 | 250 | [ONB_BACKFILL_NOTIFY_RETRYABLE] |
| ONB_BACKFILL_S10 | PH1.ONB | PH1ONB_BACKFILL_COMPLETE_COMMIT_ROW | campaign_id, tenant_id, idempotency_key | campaign_id, state, completed_target_count, total_target_count | DB_WRITE (simulation-gated) | 700 | 2 | 250 | [ONB_BACKFILL_COMPLETE_RETRYABLE] |
| ONB_BACKFILL_S11 | PH1.X | PH1X_RESPOND_COMMIT_ROW | campaign summary | completion response | DB_WRITE | 250 | 1 | 100 | [OS_RESPONSE_RETRYABLE] |

## 5) Confirmation Points
- `ONB_BACKFILL_S03` mandatory before campaign start.
- Additional confirmation required when unresolved exceptions remain at completion.

Deterministic loop note:
- `ONB_BACKFILL_S06` to `ONB_BACKFILL_S09` run per recipient in a bounded deterministic loop.
- `ONB_BACKFILL_S10` may run only after all recipient loop rows are committed or terminally exhausted.

## 6) Simulation Requirements
- `ONB_REQUIREMENT_BACKFILL_START_DRAFT`
- `ONB_REQUIREMENT_BACKFILL_NOTIFY_COMMIT`
- `ONB_REQUIREMENT_BACKFILL_COMPLETE_COMMIT`
- `BCAST_CREATE_DRAFT`
- `BCAST_DELIVER_COMMIT`
- `REMINDER_SCHEDULE_COMMIT`

## 7) Refusal Conditions
- access denied at `ONB_BACKFILL_S04` -> `ACCESS_SCOPE_VIOLATION`
- invalid campaign scope for position/schema -> `ONB_BACKFILL_SCOPE_INVALID`
- delivery policy block -> `ONB_BACKFILL_NOTIFY_BLOCKED`

## 8) Acceptance Tests
- `AT-PBS-ONBBACKFILL-01`: current-staff backfill is launched only with explicit confirmation.
- `AT-PBS-ONBBACKFILL-02`: notification/reminder path is simulation-gated and deterministic.
- `AT-PBS-ONBBACKFILL-03`: campaign completion is reason-coded with exception counts.
