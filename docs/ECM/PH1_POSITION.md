# PH1.POSITION ECM Spec

## Engine Header
- `engine_id`: `PH1.POSITION`
- `purpose`: Persist deterministic tenant-scoped position truth and append-only lifecycle transitions for create/validate/policy-check/activate/retire-suspend, plus versioned position requirements schema lifecycle.
- `data_owned`: `positions`, `position_lifecycle_events`, `position_requirements_schema_ledger`, `position_requirements_schema_current`, and PH1.POSITION-scoped `tenant_companies` writes
- `version`: `v1`
- `status`: `ACTIVE`

## Capability List

### `PH1TENANT_COMPANY_UPSERT_ROW`
- `name`: Upsert tenant-company prerequisite row
- `input_schema`: `TenantCompanyRecord`
- `output_schema`: `Result<(), StorageError>`
- `allowed_callers`: `SELENE_OS_ONLY` (simulation-gated when business onboarding commits)
- `side_effects`: `DECLARED (DB_WRITE)`

### `PH1TENANT_COMPANY_ROW`
- `name`: Read tenant-company row
- `input_schema`: `(tenant_id, company_id)`
- `output_schema`: `Option<TenantCompanyRecord>`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `NONE`

### `PH1POSITION_CREATE_DRAFT_ROW`
- `name`: Create position draft + lifecycle event
- `input_schema`: `(now, actor_user_id, tenant_id, company_id, position_title, department, jurisdiction, schedule_type, permission_profile_ref, compensation_band_ref, idempotency_key, simulation_id, reason_code)`
- `output_schema`: `Result<PositionRecord, StorageError>`
- `allowed_callers`: `SELENE_OS_ONLY` (simulation-gated)
- `side_effects`: `DECLARED (DB_WRITE)`

### `PH1POSITION_VALIDATE_AUTH_COMPANY_DRAFT_ROW`
- `name`: Validate actor/company/position authorization path
- `input_schema`: `(tenant_id, company_id, position_id, requested_action)`
- `output_schema`: `Result<(PositionValidationStatus, ReasonCodeId), StorageError>`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `NONE`

### `PH1POSITION_BAND_POLICY_CHECK_DRAFT_ROW`
- `name`: Validate compensation-band policy path
- `input_schema`: `(tenant_id, position_id, compensation_band_ref)`
- `output_schema`: `Result<(PositionPolicyResult, ReasonCodeId), StorageError>`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `NONE`

### `PH1POSITION_ACTIVATE_COMMIT_ROW`
- `name`: Activate position + append lifecycle event
- `input_schema`: `(now, actor_user_id, tenant_id, position_id, idempotency_key, simulation_id, reason_code)`
- `output_schema`: `Result<PositionRecord, StorageError>`
- `allowed_callers`: `SELENE_OS_ONLY` (simulation-gated)
- `side_effects`: `DECLARED (DB_WRITE)`

### `PH1POSITION_RETIRE_OR_SUSPEND_COMMIT_ROW`
- `name`: Retire/suspend position + append lifecycle event
- `input_schema`: `(now, actor_user_id, tenant_id, position_id, requested_state, idempotency_key, simulation_id, reason_code)`
- `output_schema`: `Result<PositionRecord, StorageError>`
- `allowed_callers`: `SELENE_OS_ONLY` (simulation-gated)
- `side_effects`: `DECLARED (DB_WRITE)`

### `PH1POSITION_REQUIREMENTS_SCHEMA_CREATE_DRAFT_ROW`
- `name`: Create position requirements schema draft
- `input_schema`: `(now, actor_user_id, tenant_id, company_id, position_id, schema_version_id, selectors, field_specs, idempotency_key, simulation_id, reason_code)`
- `output_schema`: `Result<PositionRequirementsSchemaDraftResult, StorageError>`
- `allowed_callers`: `SELENE_OS_ONLY` (simulation-gated)
- `side_effects`: `DECLARED (DB_WRITE)`

### `PH1POSITION_REQUIREMENTS_SCHEMA_UPDATE_COMMIT_ROW`
- `name`: Update position requirements schema draft
- `input_schema`: `(now, actor_user_id, tenant_id, company_id, position_id, schema_version_id, selectors, field_specs, change_reason, idempotency_key, simulation_id, reason_code)`
- `output_schema`: `Result<PositionRequirementsSchemaDraftResult, StorageError>`
- `allowed_callers`: `SELENE_OS_ONLY` (simulation-gated)
- `side_effects`: `DECLARED (DB_WRITE)`
- `determinism_rule`: `change_reason` is persisted and auditable; selector/predicate evaluation remains deterministic and fail-closed.

### `PH1POSITION_REQUIREMENTS_SCHEMA_ACTIVATE_COMMIT_ROW`
- `name`: Activate position requirements schema version
- `input_schema`: `(now, actor_user_id, tenant_id, company_id, position_id, schema_version_id, apply_scope, idempotency_key, simulation_id, reason_code)`
- `output_schema`: `Result<PositionRequirementsSchemaLifecycleResult, StorageError>`
- `allowed_callers`: `SELENE_OS_ONLY` (simulation-gated)
- `side_effects`: `DECLARED (DB_WRITE)`
- `apply_scope_rule`:
  - `apply_scope=NewHiresOnly`: activate schema for future onboarding sessions only.
  - `apply_scope=CurrentAndNew`: activate schema and emit deterministic handoff context for `ONB_REQUIREMENT_BACKFILL`.

### `PH1POSITION_ROW`
- `name`: Read one position row
- `input_schema`: `(tenant_id, position_id)`
- `output_schema`: `Option<PositionRecord>`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `NONE`

### `PH1POSITION_LIFECYCLE_ROWS_FOR_POSITION`
- `name`: Read append-only lifecycle rows for a position
- `input_schema`: `(tenant_id, position_id)`
- `output_schema`: `PositionLifecycleEventRecord[]`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `NONE`

### `PH1POSITION_APPEND_ONLY_GUARD`
- `name`: Guard against lifecycle-event overwrite
- `input_schema`: `event_id`
- `output_schema`: `Result<(), StorageError>`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `NONE`

## Failure Modes + Reason Codes
- deterministic failure domains include:
  - tenant/company scope mismatch
  - invalid lifecycle transition
  - policy/authorization validation failure
  - predicate evaluation failure for `required_rule=CONDITIONAL`
  - idempotency replay/no-op
- all failures are fail-closed and reason-coded.

## Audit Emission Requirements Per Capability
- write capabilities emit PH1.J with deterministic reason codes and bounded payload.
- write capabilities requiring side effects must remain simulation-gated (`No Simulation -> No Execution`).
- read/guard capabilities emit audit only in replay/diagnostic mode.

## Sources
- `crates/selene_storage/src/repo.rs` (`Ph1PositionRepo`)
- `docs/DB_WIRING/PH1_POSITION.md`
