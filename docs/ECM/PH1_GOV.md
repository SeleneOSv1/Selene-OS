# PH1_GOV ECM (Design vNext)

## Engine Header
- engine_id: PH1.GOV
- role: deterministic governance decision engine for artifact activation/deprecation/rollback control
- placement: ENTERPRISE_SUPPORT

## Capability List

### capability_id: GOV_POLICY_EVALUATE
- input_schema:
  - bounded envelope (`correlation_id`, `turn_id`, `max_reference_ids`, `max_diagnostics`, `enterprise_mode_signature_required`)
  - governance request metadata (`tenant_id`, `artifact_kind`, `artifact_id`, `artifact_version`, `artifact_hash_sha256`, `requested_action`)
  - requester metadata (`requester_user_id`, `requester_authorized`, optional `signature_ref`)
  - activation integrity context (`existing_active_versions`, `required_reference_ids`, `active_reference_ids`, optional `rollback_target_version`)
  - policy mode flags (`enforce_single_active_blueprint`)
- output_schema:
  - deterministic policy evaluation flags (`requester_authorized`, `signature_valid`, `references_active`, `single_active_blueprint_ok`, `rollback_target_present`)
  - invariants (`deterministic=true`, `audit_required=true`)
  - reason-coded policy result
- allowed_callers: SELENE_OS_ONLY
- side_effects: NONE
- failure_modes: INPUT_SCHEMA_INVALID, UPSTREAM_INPUT_MISSING, GOV_NOT_AUTHORIZED, GOV_SIGNATURE_INVALID, GOV_REFERENCE_MISSING, GOV_MULTI_ACTIVE_NOT_ALLOWED, BUDGET_EXCEEDED, INTERNAL_PIPELINE_ERROR
- reason_codes:
  - PH1_GOV_OK_POLICY_EVALUATE
  - GOV_NOT_AUTHORIZED
  - GOV_SIGNATURE_INVALID
  - GOV_REFERENCE_MISSING
  - GOV_MULTI_ACTIVE_NOT_ALLOWED
  - PH1_GOV_INPUT_SCHEMA_INVALID
  - PH1_GOV_UPSTREAM_INPUT_MISSING
  - PH1_GOV_BUDGET_EXCEEDED
  - PH1_GOV_INTERNAL_PIPELINE_ERROR

### capability_id: GOV_DECISION_COMPUTE
- input_schema:
  - bounded envelope (`correlation_id`, `turn_id`, `max_reference_ids`, `max_diagnostics`, `enterprise_mode_signature_required`)
  - governance action metadata (`artifact_kind`, `artifact_id`, `artifact_version`, `requested_action`)
  - current/rollback state (`current_active_version`, optional `rollback_target_version`)
  - normalized policy booleans from evaluate stage (`requester_authorized`, `signature_valid`, `references_active`, `single_active_blueprint_ok`)
  - invariants (`deterministic=true`, `audit_required=true`)
- output_schema:
  - deterministic governance decision (`ALLOWED | BLOCKED`)
  - resulting `active_version` for the requested action
  - invariants (`deterministic=true`, `audit_event_required=true`, `no_execution_authority=true`)
  - reason-coded result
- allowed_callers: SELENE_OS_ONLY
- side_effects: NONE
- failure_modes: INPUT_SCHEMA_INVALID, GOV_NOT_AUTHORIZED, GOV_SIGNATURE_INVALID, GOV_REFERENCE_MISSING, GOV_MULTI_ACTIVE_NOT_ALLOWED, INTERNAL_PIPELINE_ERROR
- reason_codes:
  - PH1_GOV_OK_DECISION_COMPUTE
  - GOV_NOT_AUTHORIZED
  - GOV_SIGNATURE_INVALID
  - GOV_REFERENCE_MISSING
  - GOV_MULTI_ACTIVE_NOT_ALLOWED
  - PH1_GOV_INPUT_SCHEMA_INVALID
  - PH1_GOV_INTERNAL_PIPELINE_ERROR

## Constraints
- Engines never call engines directly; Selene OS orchestrates all sequencing.
- PH1.GOV must never execute workflows; it only decides governance status.
- Policy and decision outputs must be deterministic and replay-safe for identical input snapshots.
- Activation decisions must fail closed when reference bindings are incomplete.

## Related Engine Boundaries
- `PH1.TENANT`: provides resolved tenant scope required before governance capability execution.
- `PBS_TABLES`: PH1.GOV decisions govern blueprint transition eligibility before registry/current updates.
- `SIMULATION_CATALOG_TABLES`: PH1.GOV governs simulation transition eligibility before catalog-current projection updates.
- `ENGINE_CAPABILITY_MAPS_TABLES`: PH1.GOV governs capability-map transition eligibility before current binding updates.
- `PH1.J`: governance decision outputs require reason-coded audit events.
