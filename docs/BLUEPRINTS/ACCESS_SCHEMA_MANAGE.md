# ACCESS_SCHEMA_MANAGE Blueprint Record

## 1) Blueprint Header
- `process_id`: `ACCESS_SCHEMA_MANAGE`
- `intent_type`: `ACCESS_SCHEMA_MANAGE`
- `version`: `v1`
- `status`: `ACTIVE`

## 1A) Contract Boundary
- This blueprint defines orchestration flow only.
- Engine behavior/schema/capability contracts are canonical in `docs/DB_WIRING/*.md` and `docs/ECM/*.md`.
- Master Access schema truth ownership remains PH1.ACCESS.001_PH2.ACCESS.002.
- Selene OS must fail closed when Access gate is not `ALLOW`.

## 2) Required Inputs
- `tenant_id` (optional for global AP operations; required for tenant scope)
- `actor_user_id`
- `access_profile_id`
- `schema_version_id`
- `ap_scope` (`GLOBAL | TENANT`)
- `ap_action` (`CREATE_DRAFT | UPDATE | ACTIVATE | RETIRE`)
- `profile_payload_json` (bounded)
- `review_channel` (`PHONE_DESKTOP | READ_OUT_LOUD`)
- `rule_review_actions` (bounded list of typed actions: `AGREE | DISAGREE | EDIT | DELETE | DISABLE | ADD_CUSTOM_RULE`)
- `change_reason`
- `idempotency_key`

## 3) Success Output Schema
```text
access_profile_id: string
schema_version_id: string
lifecycle_state: enum (DRAFT | ACTIVE | RETIRED)
scope: enum (GLOBAL | TENANT)
```

## 4) Ordered Engine Steps

| step_id | engine_name | capability_id | required_fields | produced_fields | side_effects | timeout_ms | max_retries | retry_backoff_ms | retryable_reason_codes |
|---|---|---|---|---|---|---:|---:|---:|---|
| ACCESS_SCHEMA_S01 | PH1.C | PH1C_TRANSCRIPT_OK_COMMIT_ROW | correlation_id, turn_id, transcript_hash | transcript_ok evidence row | DB_WRITE | 1200 | 1 | 150 | [STT_FAIL_PROVIDER_TIMEOUT, STT_FAIL_NETWORK_UNAVAILABLE] |
| ACCESS_SCHEMA_S02 | PH1.NLP | PH1NLP_INTENT_DRAFT_COMMIT_ROW | transcript_ok, intent_type=ACCESS_SCHEMA_MANAGE | intent_draft | DB_WRITE | 200 | 1 | 100 | [NLP_INPUT_MISSING] |
| ACCESS_SCHEMA_S03 | PH1.X | PH1X_CLARIFY_COMMIT_ROW | review_channel (if missing), access_profile_id, schema_version_id | explicit review-channel selection prompt | DB_WRITE | 300 | 1 | 100 | [OS_CLARIFY_TIMEOUT] |
| ACCESS_SCHEMA_S04 | PH1.X | PH1X_CONFIRM_COMMIT_ROW | ap_action, access_profile_id, schema_version_id, profile_payload_json, rule_review_actions | rule-review confirmation state | DB_WRITE | 300 | 1 | 100 | [OS_CONFIRM_TIMEOUT] |
| ACCESS_SCHEMA_S05 | PH1.ACCESS.001_PH2.ACCESS.002 | ACCESS_GATE_DECIDE_ROW | actor_user_id, tenant_id, requested_action=ACCESS_SCHEMA_MANAGE | access_decision | NONE | 250 | 1 | 100 | [ACCESS_SCOPE_VIOLATION] |
| ACCESS_SCHEMA_S06 | PH1.ACCESS.001_PH2.ACCESS.002 | ACCESS_AP_SCHEMA_CREATE_DRAFT_ROW | ap_action=CREATE_DRAFT, tenant_id, access_profile_id, schema_version_id, ap_scope, profile_payload_json, change_reason, idempotency_key | access_profile_id, schema_version_id, lifecycle_state=DRAFT | DB_WRITE (simulation-gated) | 700 | 2 | 250 | [ACCESS_AP_SCHEMA_INVALID] |
| ACCESS_SCHEMA_S07 | PH1.ACCESS.001_PH2.ACCESS.002 | ACCESS_AP_SCHEMA_UPDATE_COMMIT_ROW | ap_action=UPDATE, tenant_id, access_profile_id, schema_version_id, profile_payload_json, change_reason, idempotency_key | access_profile_id, schema_version_id, lifecycle_state=DRAFT | DB_WRITE (simulation-gated) | 700 | 2 | 250 | [ACCESS_AP_SCHEMA_INVALID] |
| ACCESS_SCHEMA_S08 | PH1.ACCESS.001_PH2.ACCESS.002 | ACCESS_AP_SCHEMA_ACTIVATE_COMMIT_ROW | ap_action=ACTIVATE, tenant_id, access_profile_id, schema_version_id, change_reason, idempotency_key | access_profile_id, schema_version_id, lifecycle_state=ACTIVE | DB_WRITE (simulation-gated) | 700 | 2 | 250 | [ACCESS_AP_ACTIVATION_CONFLICT] |
| ACCESS_SCHEMA_S09 | PH1.ACCESS.001_PH2.ACCESS.002 | ACCESS_AP_SCHEMA_RETIRE_COMMIT_ROW | ap_action=RETIRE, tenant_id, access_profile_id, schema_version_id, change_reason, idempotency_key | access_profile_id, schema_version_id, lifecycle_state=RETIRED | DB_WRITE (simulation-gated) | 700 | 2 | 250 | [ACCESS_AP_SCOPE_VIOLATION] |
| ACCESS_SCHEMA_S10 | PH1.X | PH1X_RESPOND_COMMIT_ROW | AP schema lifecycle result | completion response | DB_WRITE | 250 | 1 | 100 | [OS_RESPONSE_RETRYABLE] |

## 5) Confirmation Points
- `ACCESS_SCHEMA_S03` (review-channel choice) is mandatory before any AP lifecycle write.
- `ACCESS_SCHEMA_S04` (rule-by-rule review confirmation) is mandatory before any AP lifecycle write.
- Activation and retirement require explicit confirmation and reason code.
- rule actions (`AGREE | DISAGREE | EDIT | DELETE | DISABLE | ADD_CUSTOM_RULE`) must be explicitly confirmed prior to activation path.

## 6) Simulation Requirements
- `ACCESS_AP_AUTHORING_REVIEW_CHANNEL_COMMIT`
- `ACCESS_AP_AUTHORING_RULE_ACTION_COMMIT`
- `ACCESS_AP_AUTHORING_CONFIRM_COMMIT`
- `ACCESS_AP_SCHEMA_CREATE_DRAFT`
- `ACCESS_AP_SCHEMA_UPDATE_COMMIT`
- `ACCESS_AP_SCHEMA_ACTIVATE_COMMIT`
- `ACCESS_AP_SCHEMA_RETIRE_COMMIT`

## 7) Refusal Conditions
- access denied at `ACCESS_SCHEMA_S04` -> `ACCESS_SCOPE_VIOLATION`
- access escalated at `ACCESS_SCHEMA_S04` without resolved approval/override -> `ACCESS_AP_REQUIRED` (no schema lifecycle side effects)
- schema operation violates scope or activation invariants -> fail closed (`ACCESS_AP_SCOPE_VIOLATION`, `ACCESS_AP_ACTIVATION_CONFLICT`)

## 8) Acceptance Tests
- `AT-PBS-ACCESSSCHEMA-01`: AP create/update/activate/retire path remains simulation-gated and reason-coded.
- `AT-PBS-ACCESSSCHEMA-02`: Access gate must be `ALLOW` before any AP lifecycle write.
- `AT-PBS-ACCESSSCHEMA-03`: Capability IDs resolve to active ECM entries.
- `AT-PBS-ACCESSSCHEMA-04`: Simulation IDs resolve in simulation catalog.
- `AT-PBS-ACCESSSCHEMA-05`: AP authoring requires explicit review-channel choice (`PHONE_DESKTOP | READ_OUT_LOUD`) before lifecycle writes.
- `AT-PBS-ACCESSSCHEMA-06`: Rule-by-rule authoring actions are bounded and confirmed before activation.
