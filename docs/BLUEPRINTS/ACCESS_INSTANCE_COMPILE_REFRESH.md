# ACCESS_INSTANCE_COMPILE_REFRESH Blueprint Record

## 1) Blueprint Header
- `process_id`: `ACCESS_INSTANCE_COMPILE_REFRESH`
- `intent_type`: `ACCESS_INSTANCE_COMPILE_REFRESH`
- `version`: `v1`
- `status`: `ACTIVE`

## 1A) Contract Boundary
- This blueprint defines orchestration flow only.
- PH1.ACCESS.001_PH2.ACCESS.002 owns schema-chain reads and access-instance compile writes.
- Runtime gate output envelope remains unchanged (`ALLOW | DENY | ESCALATE`).
- This flow does not author AP schemas; it compiles/refreshes per-user effective access lineage.

## 2) Required Inputs
- `tenant_id`
- `actor_user_id`
- `target_user_id`
- `access_profile_id`
- `position_id` (optional)
- `overlay_id_list` (optional bounded list)
- `compile_reason`
- `idempotency_key`

## 3) Success Output Schema
```text
access_instance_id: string
compiled_global_profile_ref: string
compiled_tenant_profile_ref: string | null
compiled_overlay_set_ref: string | null
compiled_position_ref: string | null
access_mode: enum (R | W | A | X)
```

## 4) Ordered Engine Steps

| step_id | engine_name | capability_id | required_fields | produced_fields | side_effects | timeout_ms | max_retries | retry_backoff_ms | retryable_reason_codes |
|---|---|---|---|---|---|---:|---:|---:|---|
| ACCESS_COMPILE_S01 | PH1.C | PH1C_TRANSCRIPT_OK_COMMIT_ROW | correlation_id, turn_id, transcript_hash | transcript_ok evidence row | DB_WRITE | 1200 | 1 | 150 | [STT_FAIL_PROVIDER_TIMEOUT, STT_FAIL_NETWORK_UNAVAILABLE] |
| ACCESS_COMPILE_S02 | PH1.NLP | PH1NLP_INTENT_DRAFT_COMMIT_ROW | transcript_ok, intent_type=ACCESS_INSTANCE_COMPILE_REFRESH | intent_draft | DB_WRITE | 200 | 1 | 100 | [NLP_INPUT_MISSING] |
| ACCESS_COMPILE_S03 | PH1.X | PH1X_CONFIRM_COMMIT_ROW | target_user_id, access_profile_id, overlay_id_list, compile_reason | confirmation prompt state | DB_WRITE | 300 | 1 | 100 | [OS_CONFIRM_TIMEOUT] |
| ACCESS_COMPILE_S04 | PH1.ACCESS.001_PH2.ACCESS.002 | ACCESS_GATE_DECIDE_ROW | actor_user_id, tenant_id, requested_action=ACCESS_INSTANCE_COMPILE_REFRESH | access_decision | NONE | 250 | 1 | 100 | [ACCESS_SCOPE_VIOLATION] |
| ACCESS_COMPILE_S05 | PH1.ACCESS.001_PH2.ACCESS.002 | ACCESS_READ_SCHEMA_CHAIN_ROW | tenant_id, access_profile_id, overlay_id_list, position_id | resolved schema chain refs | NONE | 300 | 1 | 100 | [ACCESS_SCHEMA_REF_MISSING] |
| ACCESS_COMPILE_S06 | PH1.ACCESS.001_PH2.ACCESS.002 | ACCESS_INSTANCE_COMPILE_COMMIT_ROW | tenant_id, target_user_id, resolved schema chain refs, compile_reason, idempotency_key | access_instance_id + compiled lineage refs + access_mode | DB_WRITE (simulation-gated) | 700 | 2 | 250 | [ACCESS_PROFILE_NOT_ACTIVE] |
| ACCESS_COMPILE_S07 | PH1.X | PH1X_RESPOND_COMMIT_ROW | compile result payload | completion response | DB_WRITE | 250 | 1 | 100 | [OS_RESPONSE_RETRYABLE] |

## 5) Confirmation Points
- `ACCESS_COMPILE_S03` is mandatory before compile commit write.

## 6) Simulation Requirements
- `ACCESS_INSTANCE_COMPILE_COMMIT`

## 7) Refusal Conditions
- access denied at `ACCESS_COMPILE_S04` -> `ACCESS_SCOPE_VIOLATION`
- access escalated at `ACCESS_COMPILE_S04` without resolved approval/override -> `ACCESS_AP_REQUIRED` (no compile commit side effects)
- unresolved schema refs at `ACCESS_COMPILE_S05` -> `ACCESS_SCHEMA_REF_MISSING`
- inactive AP/overlay refs -> `ACCESS_PROFILE_NOT_ACTIVE`

## 8) Acceptance Tests
- `AT-PBS-ACCESSCOMPILE-01`: compile path writes lineage refs deterministically.
- `AT-PBS-ACCESSCOMPILE-02`: unresolved schema refs fail closed before compile commit.
- `AT-PBS-ACCESSCOMPILE-03`: capability IDs resolve to active ECM entries.
- `AT-PBS-ACCESSCOMPILE-04`: simulation IDs resolve in simulation catalog.
