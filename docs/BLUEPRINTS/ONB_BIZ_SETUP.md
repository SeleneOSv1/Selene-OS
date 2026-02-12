# ONB_BIZ_SETUP Blueprint Record

## 1) Blueprint Header
- `process_id`: `ONB_BIZ_SETUP`
- `intent_type`: `ONB_BIZ_SETUP`
- `version`: `v1`
- `status`: `DRAFT`

## 1A) Contract Boundary
- This blueprint defines orchestration flow only.
- Engine behavior/schema/capability contracts are canonical in `docs/DB_WIRING/*.md` and `docs/ECM/*.md`.

## 2) Required Inputs
- `tenant_id`
- `company_id`
- `requester_user_id`
- `idempotency_key`

## 3) Success Output Schema
```text
tenant_id: string
company_id: string
company_status: enum (ACTIVE)
business_onboarding_status: enum (COMPLETE)
```

## 4) Ordered Engine Steps

| step_id | engine_name | capability_id | required_fields | produced_fields | side_effects | timeout_ms | max_retries | retry_backoff_ms | retryable_reason_codes |
|---|---|---|---|---|---|---:|---:|---:|---|
| ONB_BIZ_S01 | PH1.C | PH1C_TRANSCRIPT_OK_COMMIT_ROW | correlation_id, turn_id, transcript_hash | transcript_ok evidence row | DB_WRITE | 1200 | 1 | 150 | [STT_FAIL_PROVIDER_TIMEOUT, STT_FAIL_NETWORK_UNAVAILABLE] |
| ONB_BIZ_S02 | PH1.NLP | PH1NLP_INTENT_DRAFT_COMMIT_ROW | transcript_ok, intent_type=ONB_BIZ_SETUP | intent_draft | DB_WRITE | 200 | 1 | 100 | [NLP_INPUT_MISSING] |
| ONB_BIZ_S03 | PH1.X | PH1X_CONFIRM_COMMIT_ROW | intent_draft | confirmation prompt state | DB_WRITE | 300 | 1 | 100 | [OS_CONFIRM_TIMEOUT] |
| ONB_BIZ_S04 | PH1.ACCESS.001_PH2.ACCESS.002 | ACCESS_GATE_DECIDE_ROW | requester_user_id, tenant_id, requested_action=ONB_BIZ_SETUP | access_decision | NONE | 250 | 1 | 100 | [ACCESS_SCOPE_VIOLATION] |
| ONB_BIZ_S05 | PH1.POSITION | PH1TENANT_COMPANY_UPSERT_ROW | tenant_id, company_id, lifecycle_state, idempotency_key | tenant_company row upserted | DB_WRITE (simulation-gated) | 700 | 2 | 250 | [POSITION_COMPANY_UPSERT_RETRYABLE] |
| ONB_BIZ_S06 | PH1.POSITION | PH1TENANT_COMPANY_ROW | tenant_id, company_id | tenant_company state snapshot | NONE | 250 | 1 | 100 | [POSITION_COMPANY_READ_RETRYABLE] |
| ONB_BIZ_S07 | PH1.X | PH1X_RESPOND_COMMIT_ROW | tenant_company state snapshot | completion response | DB_WRITE | 250 | 1 | 100 | [OS_RESPONSE_RETRYABLE] |

## 5) Confirmation Points
- `ONB_BIZ_S03` mandatory confirmation before business prerequisite commit path.

## 6) Simulation Requirements
- `ONB_BIZ_START_DRAFT`
- `ONB_BIZ_VALIDATE_COMPANY_COMMIT`
- `ONB_BIZ_COMPLETE_COMMIT`

## 7) Refusal Conditions
- Access denied at `ONB_BIZ_S04` -> `ACCESS_SCOPE_VIOLATION`
- Company prereq cannot be validated/activated -> `ONB_BIZ_COMPANY_INVALID`

## 8) Acceptance Tests
- `AT-PBS-ONBBIZ-01`: No employee onboarding completion without active company shell.
- `AT-PBS-ONBBIZ-02`: Commit path is simulation-gated.
- `AT-PBS-ONBBIZ-03`: Capability IDs resolve to active ECM entries.
