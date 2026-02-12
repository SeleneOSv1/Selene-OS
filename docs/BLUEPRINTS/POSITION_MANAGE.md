# POSITION_MANAGE Blueprint Record

## 1) Blueprint Header
- `process_id`: `POSITION_MANAGE`
- `intent_type`: `POSITION_MANAGE`
- `version`: `v1`
- `status`: `DRAFT`

## 1A) Contract Boundary
- This blueprint defines orchestration flow only.
- Engine behavior/schema/capability contracts are canonical in `docs/DB_WIRING/*.md` and `docs/ECM/*.md`.

## 2) Required Inputs
- `tenant_id`
- `company_id`
- `actor_user_id`
- `position_title`
- `department`
- `jurisdiction`
- `lifecycle_action` (`CREATE_DRAFT | ACTIVATE | SUSPEND | RETIRE`)
- `idempotency_key`

## 3) Success Output Schema
```text
tenant_id: string
position_id: string
lifecycle_state: enum (DRAFT | ACTIVE | SUSPENDED | RETIRED)
last_reason_code: string
```

## 4) Ordered Engine Steps

| step_id | engine_name | capability_id | required_fields | produced_fields | side_effects | timeout_ms | max_retries | retry_backoff_ms | retryable_reason_codes |
|---|---|---|---|---|---|---:|---:|---:|---|
| POSITION_S01 | PH1.C | PH1C_TRANSCRIPT_OK_COMMIT_ROW | correlation_id, turn_id, transcript_hash | transcript_ok evidence row | DB_WRITE | 1200 | 1 | 150 | [STT_FAIL_PROVIDER_TIMEOUT, STT_FAIL_NETWORK_UNAVAILABLE] |
| POSITION_S02 | PH1.NLP | PH1NLP_INTENT_DRAFT_COMMIT_ROW | transcript_ok, intent_type=POSITION_MANAGE | intent_draft | DB_WRITE | 200 | 1 | 100 | [NLP_INPUT_MISSING] |
| POSITION_S03 | PH1.X | PH1X_CONFIRM_COMMIT_ROW | intent_draft | confirmation prompt state | DB_WRITE | 300 | 1 | 100 | [OS_CONFIRM_TIMEOUT] |
| POSITION_S04 | PH1.ACCESS.001_PH2.ACCESS.002 | ACCESS_GATE_DECIDE_ROW | actor_user_id, tenant_id, requested_action=POSITION_MANAGE | access_decision | NONE | 250 | 1 | 100 | [ACCESS_SCOPE_VIOLATION] |
| POSITION_S05 | PH1.POSITION | PH1POSITION_CREATE_DRAFT_ROW | tenant_id, company_id, position fields, idempotency_key | position_id, lifecycle_state=DRAFT | DB_WRITE (simulation-gated) | 700 | 2 | 250 | [POSITION_CREATE_RETRYABLE] |
| POSITION_S06 | PH1.POSITION | PH1POSITION_VALIDATE_AUTH_COMPANY_DRAFT_ROW | tenant_id, company_id, position_id, requested_action | validation_status | NONE | 300 | 1 | 100 | [POSITION_VALIDATE_RETRYABLE] |
| POSITION_S07 | PH1.POSITION | PH1POSITION_BAND_POLICY_CHECK_DRAFT_ROW | tenant_id, position_id, compensation_band_ref | policy_result | NONE | 300 | 1 | 100 | [POSITION_POLICY_RETRYABLE] |
| POSITION_S08 | PH1.POSITION | PH1POSITION_ACTIVATE_COMMIT_ROW | tenant_id, position_id, actor_user_id, idempotency_key | lifecycle_state=ACTIVE | DB_WRITE (simulation-gated) | 700 | 2 | 250 | [POSITION_ACTIVATE_RETRYABLE] |
| POSITION_S09 | PH1.POSITION | PH1POSITION_RETIRE_OR_SUSPEND_COMMIT_ROW | tenant_id, position_id, requested_state, actor_user_id, idempotency_key | lifecycle_state=SUSPENDED/RETIRED | DB_WRITE (simulation-gated) | 700 | 2 | 250 | [POSITION_RETIRE_SUSPEND_RETRYABLE] |

## 5) Confirmation Points
- `POSITION_S03` mandatory confirmation before any COMMIT lifecycle transition.
- Additional confirmation before `POSITION_S08` / `POSITION_S09` when policy marks transition as high-impact.

## 6) Simulation Requirements
- `POSITION_SIM_001_CREATE_DRAFT`
- `POSITION_SIM_002_VALIDATE_AUTH_COMPANY`
- `POSITION_SIM_003_BAND_POLICY_CHECK`
- `POSITION_SIM_004_ACTIVATE_COMMIT`
- `POSITION_SIM_005_RETIRE_OR_SUSPEND_COMMIT`

## 7) Refusal Conditions
- Access denied at `POSITION_S04` -> `ACCESS_SCOPE_VIOLATION`
- Company or position scope invalid at `POSITION_S06` -> `POSITION_AUTH_COMPANY_INVALID`
- Policy check fails at `POSITION_S07` -> `POSITION_POLICY_BLOCKED`

## 8) Acceptance Tests
- `AT-PBS-POS-01`: No activation without prior create draft.
- `AT-PBS-POS-02`: Every side-effect step references a simulation.
- `AT-PBS-POS-03`: Lifecycle transitions are deterministic and reason-coded.
