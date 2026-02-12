# PH1.ACCESS.001 + PH2.ACCESS.002 ECM Spec

## Engine Header
- `engine_id`: `PH1.ACCESS.001_PH2.ACCESS.002`
- `purpose`: Persist per-user access truth and deterministic override lifecycle while exposing read-only gate decisions.
- `data_owned`: `access_instances`, `access_overrides`
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

### `ACCESS_GATE_DECIDE_ROW`
- `name`: Read-only access gate decision
- `input_schema`: `(user_id, access_engine_instance_id, requested_action, access_request_context, device_trust_level, sensitive_data_request, now)`
- `output_schema`: `Result<AccessGateDecisionRecord, StorageError>` where `AccessGateDecisionRecord` includes:
  - `access_decision` in `ALLOW | DENY | ESCALATE`
  - `escalation_trigger` (optional; includes `AP_APPROVAL_REQUIRED`)
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

## Escalation Rule (Deterministic)
- `ACCESS_GATE_DECIDE_ROW` must return `ESCALATE` (not `DENY`) when action is approvable by AP policy.
- `DENY` is valid only when no approval path exists (`ACCESS_DENY_NO_APPROVAL_PATH`).
- Selene OS orchestrates all escalation delivery and override application; this engine never triggers PH1.BCAST/PH1.DELIVERY directly.

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
  - gate decision reads emit audit only in explicit enforcement traces.

## Sources
- `crates/selene_storage/src/repo.rs` (`Ph1AccessPh2AccessRepo`)
- `docs/DB_WIRING/PH1_ACCESS_001_PH2_ACCESS_002.md`
