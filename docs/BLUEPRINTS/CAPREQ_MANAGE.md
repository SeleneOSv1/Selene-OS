# CAPREQ_MANAGE Blueprint Record

## 1) Blueprint Header
- `process_id`: `CAPREQ_MANAGE`
- `intent_type`: `CAPREQ_MANAGE`
- `version`: `v1`
- `status`: `ACTIVE`

## 2) Required Inputs
- `tenant_id`
- `requester_user_id`
- `requested_capability_id`
- `target_scope_ref`
- `justification`
- `capreq_action` (`CREATE_DRAFT | SUBMIT_FOR_APPROVAL | APPROVE | REJECT | FULFILL | CANCEL`)
- `capreq_id` (required for all non-create actions)
- `idempotency_key`

## 3) Success Output Schema
```text
capreq_id: string
status: enum (DRAFT | PENDING_APPROVAL | APPROVED | REJECTED | FULFILLED | CANCELED)
last_action: enum (CREATE_DRAFT | SUBMIT_FOR_APPROVAL | APPROVE | REJECT | FULFILL | CANCEL)
source_event_id: uint64
last_reason_code: uint64
updated_at: monotonic_time_ns
```

## 4) Ordered Engine Steps

| step_id | engine_name | capability_id | required_fields | produced_fields | side_effects | timeout_ms | max_retries | retry_backoff_ms | retryable_reason_codes |
|---|---|---|---|---|---|---:|---:|---:|---|
| CAPREQ_S01 | PH1.C | PH1C_TRANSCRIPT_OK_COMMIT_ROW | correlation_id, turn_id, transcript_hash | transcript_ok evidence row | DB_WRITE | 1200 | 1 | 150 | [STT_FAIL_PROVIDER_TIMEOUT, STT_FAIL_NETWORK_UNAVAILABLE] |
| CAPREQ_S02 | PH1.NLP | PH1NLP_INTENT_DRAFT_COMMIT_ROW | transcript_ok, intent_type=CAPREQ_MANAGE | intent_draft + parsed capreq_action/capreq_id | DB_WRITE | 200 | 1 | 100 | [NLP_INPUT_MISSING] |
| CAPREQ_S03 | PH1.X | PH1X_CONFIRM_COMMIT_ROW | intent_draft | confirmation prompt state | DB_WRITE | 300 | 1 | 100 | [OS_CONFIRM_TIMEOUT] |
| CAPREQ_S04 | PH1.ACCESS.001_PH2.ACCESS.002 | ACCESS_GATE_DECIDE_ROW | requester_user_id, tenant_id, requested_action=CAPREQ_MANAGE | access_decision | NONE | 250 | 1 | 100 | [ACCESS_SCOPE_VIOLATION] |
| CAPREQ_S05 | PH1.CAPREQ | PH1CAPREQ_CREATE_DRAFT_EXECUTE | capreq_action=CREATE_DRAFT, request payload | capreq_id, status=DRAFT | DB_WRITE (simulation-gated) | 700 | 2 | 250 | [CAPREQ_RETRYABLE] |
| CAPREQ_S06 | PH1.CAPREQ | PH1CAPREQ_SUBMIT_FOR_APPROVAL_EXECUTE | capreq_action=SUBMIT_FOR_APPROVAL, capreq_id | status=PENDING_APPROVAL | DB_WRITE (simulation-gated) | 700 | 2 | 250 | [CAPREQ_RETRYABLE] |
| CAPREQ_S07 | PH1.CAPREQ | PH1CAPREQ_APPROVE_EXECUTE | capreq_action=APPROVE, capreq_id | status=APPROVED | DB_WRITE (simulation-gated) | 700 | 2 | 250 | [CAPREQ_RETRYABLE] |
| CAPREQ_S08 | PH1.CAPREQ | PH1CAPREQ_REJECT_EXECUTE | capreq_action=REJECT, capreq_id | status=REJECTED | DB_WRITE (simulation-gated) | 700 | 2 | 250 | [CAPREQ_RETRYABLE] |
| CAPREQ_S09 | PH1.CAPREQ | PH1CAPREQ_FULFILL_EXECUTE | capreq_action=FULFILL, capreq_id | status=FULFILLED | DB_WRITE (simulation-gated) | 700 | 2 | 250 | [CAPREQ_RETRYABLE] |
| CAPREQ_S10 | PH1.CAPREQ | PH1CAPREQ_CANCEL_REVOKE_EXECUTE | capreq_action=CANCEL, capreq_id | status=CANCELED | DB_WRITE (simulation-gated) | 700 | 2 | 250 | [CAPREQ_RETRYABLE] |

## 5) Confirmation Points
- `CAPREQ_S03` mandatory pre-submit/pre-transition confirmation for impactful actions.
- Additional confirmation before `CAPREQ_S10` cancel path.

## 6) Simulation Requirements
- `CAPREQ_CREATE_DRAFT`
- `CAPREQ_SUBMIT_FOR_APPROVAL_COMMIT`
- `CAPREQ_APPROVE_COMMIT`
- `CAPREQ_REJECT_COMMIT`
- `CAPREQ_FULFILL_COMMIT`
- `CAPREQ_CANCEL_REVOKE`

## 7) Refusal Conditions
- Access denied at `CAPREQ_S04` -> `ACCESS_SCOPE_VIOLATION`
- Invalid lifecycle transition for requested action -> `CAPREQ_TRANSITION_INVALID`
- Missing `capreq_id` for non-create actions -> `CAPREQ_ID_REQUIRED`

## 8) Acceptance Tests
- `AT-PBS-CAPREQ-01`: Lifecycle transitions only through declared simulations.
- `AT-PBS-CAPREQ-02`: Invalid transitions fail closed with deterministic reason code.
- `AT-PBS-CAPREQ-03`: Capability IDs in steps resolve to active ECM entries.
- `AT-PBS-CAPREQ-04`: One active process for `intent_type=CAPREQ_MANAGE` in registry.
