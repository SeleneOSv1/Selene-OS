# PH1.ACCESS.001 + PH2.ACCESS.002 ECM Spec

## Engine Header
- `engine_id`: `PH1.ACCESS.001_PH2.ACCESS.002`
- `purpose`: Persist schema-driven master-access truth (AP versions/overlays/board policy) plus per-user access truth and deterministic override lifecycle while exposing read-only gate decisions.
- `data_owned`: `access_instances`, `access_overrides`, `access_ap_schemas_ledger`, `access_ap_schemas_current`, `access_ap_overlay_ledger`, `access_ap_overlay_current`, `access_board_policy_ledger`, `access_board_policy_current`, `access_board_votes_ledger`
- `version`: `v1`
- `status`: `ACTIVE`

## Capability List

### `ACCESS_UPSERT_INSTANCE_COMMIT_ROW`
- `name`: Upsert per-user access instance
- `input_schema`: `(now, tenant_id, user_id, role_template_id, effective_access_mode, baseline_permissions_json, identity_verified, verification_level, device_trust_level, lifecycle_state, policy_snapshot_ref, idempotency_key)`
- `output_schema`: `Result<AccessInstanceRecord, StorageError>`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `DECLARED (DB_WRITE)`

### `ACCESS_APPLY_OVERRIDE_COMMIT_ROW`
- `name`: Append override lifecycle row
- `input_schema`: `(now, tenant_id, access_instance_id, override_type, scope_json, approved_by_user_id, approved_via_simulation_id, reason_code, starts_at, expires_at, idempotency_key)`
- `output_schema`: `Result<AccessOverrideRecord, StorageError>`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `DECLARED (DB_WRITE)`

### `ACCESS_AP_SCHEMA_CREATE_DRAFT_ROW`
- `name`: Append AP schema draft row
- `input_schema`: `(now, tenant_id|null, access_profile_id, schema_version_id, scope, profile_payload_json, reason_code, created_by_user_id, idempotency_key)`
- `output_schema`: `Result<AccessProfileSchemaRecord, StorageError>`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `DECLARED (DB_WRITE)`

### `ACCESS_AP_SCHEMA_UPDATE_COMMIT_ROW`
- `name`: Append AP schema update row
- `input_schema`: `(now, tenant_id|null, access_profile_id, schema_version_id, update_payload_json, reason_code, created_by_user_id, idempotency_key)`
- `output_schema`: `Result<AccessProfileSchemaRecord, StorageError>`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `DECLARED (DB_WRITE)`

### `ACCESS_AP_SCHEMA_ACTIVATE_COMMIT_ROW`
- `name`: Activate AP schema version in current projection
- `input_schema`: `(now, tenant_id|null, access_profile_id, schema_version_id, reason_code, created_by_user_id, idempotency_key)`
- `output_schema`: `Result<AccessProfileSchemaRecord, StorageError>`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `DECLARED (DB_WRITE)`

### `ACCESS_AP_SCHEMA_RETIRE_COMMIT_ROW`
- `name`: Retire AP schema version
- `input_schema`: `(now, tenant_id|null, access_profile_id, schema_version_id, reason_code, created_by_user_id, idempotency_key)`
- `output_schema`: `Result<AccessProfileSchemaRecord, StorageError>`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `DECLARED (DB_WRITE)`

### `ACCESS_AP_OVERLAY_UPDATE_COMMIT_ROW`
- `name`: Append overlay lifecycle row and update current projection
- `input_schema`: `(now, tenant_id, overlay_id, overlay_version_id, event_action, overlay_ops_json, reason_code, created_by_user_id, idempotency_key)`
- `output_schema`: `Result<AccessOverlayRecord, StorageError>`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `DECLARED (DB_WRITE)`

### `ACCESS_BOARD_POLICY_UPDATE_COMMIT_ROW`
- `name`: Append board policy lifecycle row and update current projection
- `input_schema`: `(now, tenant_id, board_policy_id, policy_version_id, event_action, policy_payload_json, reason_code, created_by_user_id, idempotency_key)`
- `output_schema`: `Result<AccessBoardPolicyRecord, StorageError>`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `DECLARED (DB_WRITE)`

### `ACCESS_BOARD_VOTE_COMMIT_ROW`
- `name`: Append board vote row for escalation case
- `input_schema`: `(now, tenant_id, escalation_case_id, board_policy_id, voter_user_id, vote_value, reason_code, idempotency_key)`
- `output_schema`: `Result<AccessBoardVoteRecord, StorageError>`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `DECLARED (DB_WRITE)`

### `ACCESS_INSTANCE_COMPILE_COMMIT_ROW`
- `name`: Compile and persist effective access instance with schema lineage refs
- `input_schema`: `(now, tenant_id, user_id, role_template_id, compile_chain_refs, effective_permissions_json, effective_access_mode, identity_verified, verification_level, device_trust_level, lifecycle_state, policy_snapshot_ref, idempotency_key)`
- `output_schema`: `Result<AccessInstanceRecord, StorageError>`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `DECLARED (DB_WRITE)`

### `ACCESS_GATE_DECIDE_ROW`
- `name`: Read-only access gate decision
- `input_schema`: `(user_id, access_engine_instance_id, requested_action, access_request_context, device_trust_level, sensitive_data_request, now)`
- `output_schema`: `Result<AccessGateDecisionRecord, StorageError>` where `AccessGateDecisionRecord` includes:
  - `access_decision` in `ALLOW | DENY | ESCALATE`
  - `escalation_trigger` (optional; includes `AP_APPROVAL_REQUIRED` and `SMS_APP_SETUP_REQUIRED`)
  - `required_approver_selector` (optional)
  - `requested_scope` (optional)
  - `requested_duration` (optional)
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `NONE`

### `ACCESS_READ_INSTANCE_AND_OVERRIDES`
- `name`: Read instance and override state
- `input_schema`: `tenant_id + user_id | access_instance_id`
- `output_schema`: `AccessInstanceRecord | AccessOverrideRecord[]`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `NONE`

### `ACCESS_READ_SCHEMA_CHAIN_ROW`
- `name`: Read active AP/overlay/board schema chain for deterministic access evaluation
- `input_schema`: `(tenant_id, access_profile_id, overlay_id[], board_policy_id?)`
- `output_schema`: `(global_ap_version, tenant_ap_version?, active_overlays[], active_board_policy?)`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `NONE`

### `ACCESS_APPEND_ONLY_GUARD`
- `name`: Attempt overwrite override guard (must fail)
- `input_schema`: `override_id`
- `output_schema`: `Result<(), StorageError>`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `NONE`

## Failure Modes + Reason Codes
- append-only mutation attempt: `ACCESS_APPEND_ONLY_VIOLATION`
- idempotency replay/no-op: `ACCESS_IDEMPOTENCY_REPLAY`
- tenant/user scope mismatch: `ACCESS_SCOPE_VIOLATION`
- contract validation failure: `ACCESS_CONTRACT_VALIDATION_FAILED`
- policy denies all approval paths: `ACCESS_DENY_NO_APPROVAL_PATH`
- SMS setup required before SMS send path: `ACCESS_SMS_SETUP_REQUIRED`
- schema reference missing in compile/eval chain: `ACCESS_SCHEMA_REF_MISSING`
- AP version not active for decision time: `ACCESS_PROFILE_NOT_ACTIVE`
- overlay reference invalid for tenant scope: `ACCESS_OVERLAY_REF_INVALID`
- board policy payload invalid: `ACCESS_BOARD_POLICY_INVALID`
- board voter not authorized: `ACCESS_BOARD_MEMBER_REQUIRED`

## Escalation Rule (Deterministic)
- Effective permission chain resolution order is fixed:
  - global AP version -> tenant AP version (if present) -> tenant overlays -> position-local rules -> active per-user overrides.
- If any required schema ref in the chain is missing/invalid/not active, return `DENY` (`ACCESS_SCHEMA_REF_MISSING` or `ACCESS_PROFILE_NOT_ACTIVE`).
- `ACCESS_GATE_DECIDE_ROW` must return `ESCALATE` (not `DENY`) when action is approvable by AP policy.
- `ACCESS_GATE_DECIDE_ROW` must return `ESCALATE` when SMS delivery is requested and `sms_app_setup_complete=false` (`ACCESS_SMS_SETUP_REQUIRED`).
- `ACCESS_GATE_DECIDE_ROW` must return `ESCALATE` when approval threshold policy exists but required votes/approvals are not yet satisfied.
- `DENY` is valid only when no approval path exists (`ACCESS_DENY_NO_APPROVAL_PATH`).
- Selene OS orchestrates all escalation delivery and override application; this engine never triggers PH1.BCAST/PH1.DELIVERY directly.
- callers must treat non-allow decisions as non-executable for governed commits:
  - `DENY` -> fail closed, no governed commit
  - `ESCALATE` -> fail closed pending approval/override resolution

## Audit Emission Requirements Per Capability
- Write capabilities must emit PH1.J events with:
  - `event_type`
  - `reason_code`
  - `tenant_id`
  - `user_id` or `access_instance_id`
  - `idempotency_key`
- Minimum event classes:
  - instance upsert: `STATE_TRANSITION`
  - override apply: `STATE_TRANSITION`
  - AP schema lifecycle writes: `STATE_TRANSITION`
  - overlay lifecycle writes: `STATE_TRANSITION`
  - board policy/vote writes: `STATE_TRANSITION`
  - access instance compile writes: `STATE_TRANSITION`
  - gate decision reads emit audit only in explicit enforcement traces.

## Sources
- `crates/selene_storage/src/repo.rs` (`Ph1AccessPh2AccessRepo`)
- `docs/DB_WIRING/PH1_ACCESS_001_PH2_ACCESS_002.md`
