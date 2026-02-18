# PH1_KMS ECM (Design vNext)

## Engine Header
- engine_id: PH1.KMS
- role: Secret/key access evaluation and opaque handle issuance
- placement: ENTERPRISE_SUPPORT

## Capability List

### capability_id: KMS_ACCESS_EVALUATE
- input_schema:
  - bounded envelope (`correlation_id`, `turn_id`, `max_allowlist_entries`, `max_diagnostics`)
  - tenant + secret request context (`tenant_id`, `secret_name`, `operation`)
  - requester context (`requester_engine_id`, optional `requester_user_id`, allowlist)
  - operation controls (`requested_ttl_ms`, `require_admin_for_rotation`, `now_ms`)
- output_schema:
  - operation echo
  - opaque `secret_ref`
  - optional resolved TTL for ephemeral requests
  - security flags (`requester_authorized=true`, `no_secret_value_emitted=true`, `audit_safe=true`)
- allowed_callers: SELENE_OS_ONLY
- side_effects: NONE
- failure_modes: INPUT_SCHEMA_INVALID, UPSTREAM_INPUT_MISSING, KMS_NOT_AUTHORIZED, KMS_SECRET_NOT_FOUND, KMS_TTL_OUT_OF_BOUNDS, INTERNAL_PIPELINE_ERROR
- reason_codes:
  - PH1_KMS_OK_ACCESS_EVALUATE
  - PH1_KMS_INPUT_SCHEMA_INVALID
  - PH1_KMS_UPSTREAM_INPUT_MISSING
  - KMS_NOT_AUTHORIZED
  - KMS_SECRET_NOT_FOUND
  - KMS_TTL_OUT_OF_BOUNDS
  - PH1_KMS_INTERNAL_PIPELINE_ERROR

### capability_id: KMS_MATERIAL_ISSUE
- input_schema:
  - bounded envelope (`correlation_id`, `turn_id`, `max_allowlist_entries`, `max_diagnostics`)
  - operation + `secret_ref` from `KMS_ACCESS_EVALUATE`
  - requester context (`requester_engine_id`, optional `requester_user_id`)
  - operation-shape fields (`resolved_ttl_ms`, optional `previous_version`, `require_no_secret_value_emission`)
- output_schema:
  - validation result (`OK|FAIL`) + bounded diagnostics
  - operation-shaped output:
    - `GET_HANDLE` -> `secret_handle`
    - `ISSUE_EPHEMERAL` -> `ephemeral_credential_ref`
    - `ROTATE` -> `secret_handle` + `rotated_version`
    - `REVOKE` -> revoked tombstone handle metadata
  - security flags (`no_secret_value_emitted=true`, `audit_safe=true`)
- allowed_callers: SELENE_OS_ONLY
- side_effects: NONE
- failure_modes: VALIDATION_FAILED, KMS_ROTATION_FAILED, INPUT_SCHEMA_INVALID, KMS_NOT_AUTHORIZED, INTERNAL_PIPELINE_ERROR
- reason_codes:
  - PH1_KMS_OK_MATERIAL_ISSUE
  - PH1_KMS_VALIDATION_FAILED
  - KMS_ROTATION_FAILED
  - PH1_KMS_INPUT_SCHEMA_INVALID
  - KMS_NOT_AUTHORIZED
  - PH1_KMS_INTERNAL_PIPELINE_ERROR

## Constraints
- Engines never call engines directly; Selene OS orchestrates all sequencing.
- Must never emit secret values; outputs are opaque references only.
- Authorization is deterministic and fail-closed.
- Rotation/revoke actions require admin constraints when configured.
- KMS material outputs never grant execution authority by themselves.

## Related Engine Boundaries
- PH1.J logging boundary: KMS events must include reason codes and opaque references only.
- PH1.EXPORT boundary: no raw secret material may appear in compliance exports.
- PH1.WORK/PH1.OS retry boundary: operation retries must remain idempotent and deterministic via opaque refs.
