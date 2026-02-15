# ACCESS_ESCALATION_VOTE Blueprint Record

## 1) Blueprint Header
- `process_id`: `ACCESS_ESCALATION_VOTE`
- `intent_type`: `ACCESS_ESCALATION_VOTE`
- `version`: `v1`
- `status`: `ACTIVE`

## 1A) Contract Boundary
- This blueprint defines orchestration flow only.
- Access authority decisions remain in PH1.ACCESS.001_PH2.ACCESS.002.
- PH1.BCAST and PH1.REM handle notification/reminder delivery only.
- No governed side effects execute until approval threshold is deterministically satisfied.

## 2) Required Inputs
- `tenant_id`
- `actor_user_id` (requester)
- `escalation_case_id`
- `board_policy_id`
- `target_user_id`
- `access_instance_id`
- `requested_action`
- `vote_action` (`CAST_VOTE | RESOLVE`)
- `vote_value` (`APPROVE | REJECT`; required for CAST_VOTE)
- `override_result` (`ONE_SHOT | TEMPORARY | PERMANENT | DENY`; required for RESOLVE)
- `idempotency_key`

## 3) Success Output Schema
```text
escalation_case_id: string
threshold_status: enum (PENDING | SATISFIED | REJECTED)
override_id: string | null
gate_ready: boolean
```

## 4) Ordered Engine Steps

| step_id | engine_name | capability_id | required_fields | produced_fields | side_effects | timeout_ms | max_retries | retry_backoff_ms | retryable_reason_codes |
|---|---|---|---|---|---|---:|---:|---:|---|
| ACCESS_VOTE_S01 | PH1.C | PH1C_TRANSCRIPT_OK_COMMIT_ROW | correlation_id, turn_id, transcript_hash | transcript_ok evidence row | DB_WRITE | 1200 | 1 | 150 | [STT_FAIL_PROVIDER_TIMEOUT, STT_FAIL_NETWORK_UNAVAILABLE] |
| ACCESS_VOTE_S02 | PH1.NLP | PH1NLP_INTENT_DRAFT_COMMIT_ROW | transcript_ok, intent_type=ACCESS_ESCALATION_VOTE | intent_draft | DB_WRITE | 200 | 1 | 100 | [NLP_INPUT_MISSING] |
| ACCESS_VOTE_S03 | PH1.X | PH1X_CONFIRM_COMMIT_ROW | vote_action, escalation_case_id, vote_value or override_result | confirmation prompt state | DB_WRITE | 300 | 1 | 100 | [OS_CONFIRM_TIMEOUT] |
| ACCESS_VOTE_S04 | PH1.ACCESS.001_PH2.ACCESS.002 | ACCESS_GATE_DECIDE_ROW | actor_user_id, tenant_id, requested_action=ACCESS_ESCALATION_VOTE | access_decision | NONE | 250 | 1 | 100 | [ACCESS_SCOPE_VIOLATION] |
| ACCESS_VOTE_S05 | PH1.ACCESS.001_PH2.ACCESS.002 | ACCESS_BOARD_VOTE_COMMIT_ROW | vote_action=CAST_VOTE, tenant_id, escalation_case_id, board_policy_id, actor_user_id, vote_value, idempotency_key | escalation_case_id, threshold_status | DB_WRITE (simulation-gated) | 700 | 2 | 250 | [ACCESS_BOARD_MEMBER_REQUIRED] |
| ACCESS_VOTE_S06 | PH1.ACCESS.001_PH2.ACCESS.002 | ACCESS_APPLY_OVERRIDE_COMMIT_ROW | vote_action=RESOLVE, threshold_status=SATISFIED, target_user_id, access_instance_id, override_result, idempotency_key | override_id, override_status | DB_WRITE (simulation-gated) | 700 | 2 | 250 | [ACCESS_AP_REQUIRED] |
| ACCESS_VOTE_S07 | PH1.ACCESS.001_PH2.ACCESS.002 | ACCESS_GATE_DECIDE_ROW | actor_user_id=target_user_id, tenant_id, requested_action | gate_ready decision | NONE | 250 | 1 | 100 | [ACCESS_SCOPE_VIOLATION] |
| ACCESS_VOTE_S08 | PH1.X | PH1X_RESPOND_COMMIT_ROW | threshold result + optional override result + gate_ready | completion response | DB_WRITE | 250 | 1 | 100 | [OS_RESPONSE_RETRYABLE] |

## 5) Confirmation Points
- `ACCESS_VOTE_S03` is mandatory for vote cast and resolution actions.
- Resolution that applies override requires explicit confirmation.

## 6) Simulation Requirements
- `ACCESS_BOARD_VOTE_COMMIT`
- `ACCESS_OVERRIDE_TEMP_GRANT_COMMIT`
- `ACCESS_OVERRIDE_PERM_GRANT_COMMIT`
- `ACCESS_OVERRIDE_REVOKE_COMMIT`

## 7) Refusal Conditions
- access denied at `ACCESS_VOTE_S04` -> `ACCESS_SCOPE_VIOLATION`
- access escalated at `ACCESS_VOTE_S04` without resolved approval for vote action -> `ACCESS_AP_REQUIRED`
- board threshold not satisfied at resolve time -> fail closed (`ACCESS_BOARD_POLICY_INVALID`)
- non-board member vote attempt -> fail closed (`ACCESS_BOARD_MEMBER_REQUIRED`)

## 8) Acceptance Tests
- `AT-PBS-ACCESSVOTE-01`: board vote rows are append-only and idempotent by `(tenant_id, escalation_case_id, voter_user_id, idempotency_key)`.
- `AT-PBS-ACCESSVOTE-02`: override apply path runs only after threshold is satisfied.
- `AT-PBS-ACCESSVOTE-03`: capability IDs resolve to active ECM entries.
- `AT-PBS-ACCESSVOTE-04`: simulation IDs resolve in simulation catalog.
