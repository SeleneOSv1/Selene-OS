# PH1.POSITION DB Wiring Spec

## 1) Engine Header

- `engine_id`: `PH1.POSITION`
- `purpose`: Persist deterministic tenant-scoped position truth (`Draft | Active | Suspended | Retired`) and append-only lifecycle transitions for onboarding/invite flows, plus versioned position requirements schemas used by PH1.ONB execution.
- `version`: `v1`
- `status`: `PASS`

## 2) Data Owned (authoritative)

### `positions`
- `truth_type`: `CURRENT`
- `primary key`: `(tenant_id, position_id)`
- invariants:
  - one current record per tenant-scoped position
  - lifecycle state transitions are deterministic and bounded
  - create uniqueness is tenant/company/title/department/jurisdiction scoped

### `position_lifecycle_events`
- `truth_type`: `LEDGER`
- `primary key`: `event_id`
- invariants:
  - append-only lifecycle history for each `(tenant_id, position_id)`
  - each transition carries simulation + reason code + actor
  - idempotent retries must not duplicate effective transitions

### Position requirements schema records
- `truth_type`: `LEDGER + CURRENT`
- ownership:
  - PH1.POSITION owns versioned requirements schema definitions attached to a `position_id`
  - PH1.POSITION owns schema overlays/selectors attached to a `position_id`
- invariants:
  - active schema version is unique per `(tenant_id, position_id)`
  - activation is monotonic and auditable (no silent in-place mutation)
  - schema update commits persist `change_reason` as auditable mutation intent
  - `apply_scope` semantics are deterministic and auditable:
    - `NewHiresOnly`: activation applies to future onboarding sessions only
    - `CurrentAndNew`: activation applies to new sessions and requires explicit backfill orchestration
  - required-rule evaluation is deterministic and fail-closed:
    - `ALWAYS`: field is required
    - `CONDITIONAL`: predicate reference must be evaluated against bounded selector snapshot
  - selector evaluation and overlay merge order are deterministic and bounded
  - every schema activation is simulation-gated and reason-coded
  - PH1.ONB consumes these records read-only; PH1.ONB never mutates schema truth

### `tenant_companies` (dependency for position authorization/validity)
- `truth_type`: `CURRENT`
- `primary key`: `(tenant_id, company_id)`
- invariants:
  - position create/validation requires company to exist and be `ACTIVE`

## 3) Reads (dependencies)

- identity prerequisite:
  - actor user must exist in `identities` for create/activate/retire-suspend commits
- company prerequisite:
  - `tenant_companies(tenant_id, company_id)` must exist and be `ACTIVE` for create/validation
- scoped position read:
  - all reads/writes are keyed by `(tenant_id, position_id)`; cross-tenant access returns not found
- schema read/write prerequisite:
  - schema draft/update/activate operations require tenant-matched position scope and actor authorization
  - schema update/activate operations require `positions.lifecycle_state=ACTIVE` for the target position
- access/approval prerequisite for governed writes:
  - Selene OS must resolve Access gate decision before governed POSITION commits (including requirements-schema lifecycle commits)
  - `ALLOW` permits commit execution
  - `DENY` and `ESCALATE` are fail-closed (no POSITION commit write until approval/override path resolves)

## 4) Writes (outputs)

### `POSITION_SIM_001_CREATE_DRAFT`
- writes: `positions` current row + append `position_lifecycle_events` (CreateDraft)
- idempotency key rule:
  - dedupe by `(tenant_id, company_id, position_title, department, jurisdiction, idempotency_key)`

### `POSITION_SIM_002_VALIDATE_AUTH_COMPANY` (draft check)
- writes: none (read-only validation result)

### `POSITION_SIM_003_BAND_POLICY_CHECK` (draft check)
- writes: none (read-only policy result)

### `POSITION_SIM_004_ACTIVATE_COMMIT`
- writes: update `positions.lifecycle_state` + append lifecycle event (Activate)
- idempotency key rule:
  - dedupe by `(tenant_id, position_id, idempotency_key)`

### `POSITION_SIM_005_RETIRE_OR_SUSPEND_COMMIT`
- writes: update `positions.lifecycle_state` + append lifecycle event (Suspend/Retire)
- idempotency key rule:
  - dedupe by `(tenant_id, position_id, requested_state, idempotency_key)`

### Position requirements schema lifecycle writes
- writes:
  - create requirements schema draft for position
  - update requirements schema draft (add/remove/override fields and conditional rules) and persist `change_reason`
  - activate new schema version for position with explicit `apply_scope` (`NewHiresOnly | CurrentAndNew`)
  - when `apply_scope=CurrentAndNew`, emit deterministic handoff context for `ONB_REQUIREMENT_BACKFILL`
- simulation bindings:
  - `POSITION_REQUIREMENTS_SCHEMA_CREATE_DRAFT`
  - `POSITION_REQUIREMENTS_SCHEMA_UPDATE_COMMIT`
  - `POSITION_REQUIREMENTS_SCHEMA_ACTIVATE_COMMIT`
- idempotency key rule:
  - dedupe by `(tenant_id, position_id, schema_version_id, action, idempotency_key)`

## 5) Relations & Keys

- `positions(tenant_id, company_id)` references `tenant_companies(tenant_id, company_id)` contract scope.
- lifecycle rows are tenant-position scoped and never mutated in place.
- position requirements schema records are keyed by `(tenant_id, position_id, schema_version)` and linked to position lifecycle scope.
- PH1.ONB session pinning references active position schema version and effective overlays at session start.
- when `apply_scope=CurrentAndNew`, PH1.ONB backfill campaign flow is launched as an explicit, simulation-gated process (`ONB_REQUIREMENT_BACKFILL`).

## 6) Audit/Proof Emissions

Row 22 lock centers on PH1.POSITION current + lifecycle + requirements-schema persistence.
Append-only evidence is provided by `position_lifecycle_events` and schema activation audit emissions; overwrite attempts must fail deterministically.

## 7) Acceptance Tests (DB Wiring Proof)

- `AT-POSITION-DB-01` tenant isolation enforced
  - `at_position_db_01_tenant_isolation_enforced`
- `AT-POSITION-DB-02` append-only enforced
  - `at_position_db_02_append_only_enforced`
- `AT-POSITION-DB-03` idempotency dedupe works
  - `at_position_db_03_idempotency_dedupe_works`
- `AT-POSITION-DB-04` current table consistent with lifecycle ledger
  - `at_position_db_04_current_table_consistency_with_lifecycle_ledger`
- `AT-POSITION-DB-05` schema activation monotonicity and replay safety
  - `at_position_db_05_requirements_schema_activation_monotonic`
- `AT-POSITION-DB-06` PH1.ONB reads active position schema but cannot mutate it
  - `at_position_db_06_onb_read_only_schema_boundary`

Implementation references:
- storage wiring: `crates/selene_storage/src/ph1f.rs`
- typed repo: `crates/selene_storage/src/repo.rs`
- tests: `crates/selene_storage/tests/ph1_position/db_wiring.rs`
