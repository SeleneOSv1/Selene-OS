# ONB_SCHEMA_MANAGE Blueprint Record

## 1) Blueprint Header
- `process_id`: `ONB_SCHEMA_MANAGE`
- `intent_type`: `ONB_SCHEMA_MANAGE`
- `version`: `v1`
- `status`: `ACTIVE`

## 1A) Contract Boundary
- This blueprint defines orchestration flow only.
- Engine behavior/schema/capability contracts are canonical in `docs/DB_WIRING/*.md` and `docs/ECM/*.md`.

## 2) Required Inputs
- `tenant_id`
- `actor_user_id`
- `company_id`
- `position_id`
- `schema_version_id`
- `selectors` (bounded selector snapshot)
- `field_specs` (typed requirement field definitions)
- `change_reason` (required for update commit)
- `apply_scope` (`NEW_HIRES_ONLY | CURRENT_AND_NEW`)
- `idempotency_key`

## 3) Success Output Schema
```text
position_id: string
schema_version_id: string
field_count: integer (create/update)
active_for_new_hires: boolean (activate)
apply_scope: enum (NEW_HIRES_ONLY | CURRENT_AND_NEW)
```

## 4) Ordered Engine Steps

| step_id | engine_name | capability_id | required_fields | produced_fields | side_effects | timeout_ms | max_retries | retry_backoff_ms | retryable_reason_codes |
|---|---|---|---|---|---|---:|---:|---:|---|
| ONB_SCHEMA_S01 | PH1.C | PH1C_TRANSCRIPT_OK_COMMIT_ROW | correlation_id, turn_id, transcript_hash | transcript_ok evidence row | DB_WRITE | 1200 | 1 | 150 | [STT_FAIL_PROVIDER_TIMEOUT, STT_FAIL_NETWORK_UNAVAILABLE] |
| ONB_SCHEMA_S02 | PH1.NLP | PH1NLP_INTENT_DRAFT_COMMIT_ROW | transcript_ok, intent_type=ONB_SCHEMA_MANAGE | intent_draft | DB_WRITE | 200 | 1 | 100 | [NLP_INPUT_MISSING] |
| ONB_SCHEMA_S03 | PH1.X | PH1X_CLARIFY_COMMIT_ROW | missing schema selectors/field ops (if any) | one-question clarify state | DB_WRITE | 300 | 1 | 100 | [OS_CLARIFY_TIMEOUT] |
| ONB_SCHEMA_S04 | PH1.X | PH1X_CONFIRM_COMMIT_ROW | selectors, field_specs, apply_scope | confirmation prompt state | DB_WRITE | 300 | 1 | 100 | [OS_CONFIRM_TIMEOUT] |
| ONB_SCHEMA_S05 | PH1.ACCESS.001_PH2.ACCESS.002 | ACCESS_GATE_DECIDE_ROW | actor_user_id, tenant_id, requested_action=ONB_SCHEMA_MANAGE | access_decision | NONE | 250 | 1 | 100 | [ACCESS_SCOPE_VIOLATION] |
| ONB_SCHEMA_S06 | PH1.POSITION | PH1POSITION_REQUIREMENTS_SCHEMA_CREATE_DRAFT_ROW | actor_user_id, tenant_id, company_id, position_id, schema_version_id, selectors, field_specs, idempotency_key | position_id, schema_version_id, field_count | DB_WRITE (simulation-gated) | 700 | 2 | 250 | [POSITION_SCHEMA_CREATE_RETRYABLE] |
| ONB_SCHEMA_S07 | PH1.POSITION | PH1POSITION_REQUIREMENTS_SCHEMA_UPDATE_COMMIT_ROW | actor_user_id, tenant_id, company_id, position_id, schema_version_id, selectors, field_specs, change_reason, idempotency_key | position_id, schema_version_id, field_count | DB_WRITE (simulation-gated) | 700 | 2 | 250 | [POSITION_SCHEMA_UPDATE_RETRYABLE] |
| ONB_SCHEMA_S08 | PH1.POSITION | PH1POSITION_REQUIREMENTS_SCHEMA_ACTIVATE_COMMIT_ROW | actor_user_id, tenant_id, company_id, position_id, schema_version_id, apply_scope, idempotency_key | position_id, schema_version_id, active_for_new_hires | DB_WRITE (simulation-gated) | 700 | 2 | 250 | [POSITION_SCHEMA_ACTIVATE_RETRYABLE] |
| ONB_SCHEMA_S09 | PH1.X | PH1X_RESPOND_COMMIT_ROW | schema activation result | completion response | DB_WRITE | 250 | 1 | 100 | [OS_RESPONSE_RETRYABLE] |

## 5) Confirmation Points
- `ONB_SCHEMA_S04` mandatory before any schema commit/activation step.
- Additional confirmation is required when `apply_scope=CURRENT_AND_NEW`.

## 6) Simulation Requirements
- `POSITION_REQUIREMENTS_SCHEMA_CREATE_DRAFT`
- `POSITION_REQUIREMENTS_SCHEMA_UPDATE_COMMIT`
- `POSITION_REQUIREMENTS_SCHEMA_ACTIVATE_COMMIT`

## 7) Refusal Conditions
- access denied at `ONB_SCHEMA_S05` -> `ACCESS_SCOPE_VIOLATION`
- schema operation invalid for position scope -> `POSITION_SCHEMA_SCOPE_INVALID`
- schema activation policy blocked -> `POSITION_SCHEMA_POLICY_BLOCKED`

## 8) Acceptance Tests
- `AT-PBS-ONBSCHEMA-01`: position requirements schema changes are versioned and simulation-gated.
- `AT-PBS-ONBSCHEMA-02`: activation requires explicit confirmation and access allow.
- `AT-PBS-ONBSCHEMA-03`: capability IDs resolve to active ECM entries.
