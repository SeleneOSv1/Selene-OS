# PH1.POSITION DB Wiring Spec

## 1) Engine Header

- `engine_id`: `PH1.POSITION`
- `purpose`: Persist deterministic tenant-scoped position truth (`Draft | Active | Suspended | Retired`) and append-only lifecycle transitions for onboarding/invite flows.
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

## 5) Relations & Keys

- `positions(tenant_id, company_id)` references `tenant_companies(tenant_id, company_id)` contract scope.
- lifecycle rows are tenant-position scoped and never mutated in place.

## 6) Audit/Proof Emissions

Row 22 lock centers on PH1.POSITION current + lifecycle persistence.
Append-only evidence is provided by `position_lifecycle_events`; overwrite attempts must fail deterministically.

## 7) Acceptance Tests (DB Wiring Proof)

- `AT-POSITION-DB-01` tenant isolation enforced
  - `at_position_db_01_tenant_isolation_enforced`
- `AT-POSITION-DB-02` append-only enforced
  - `at_position_db_02_append_only_enforced`
- `AT-POSITION-DB-03` idempotency dedupe works
  - `at_position_db_03_idempotency_dedupe_works`
- `AT-POSITION-DB-04` current table consistent with lifecycle ledger
  - `at_position_db_04_current_table_consistency_with_lifecycle_ledger`

Implementation references:
- storage wiring: `crates/selene_storage/src/ph1f.rs`
- typed repo: `crates/selene_storage/src/repo.rs`
- tests: `crates/selene_storage/tests/ph1_position/db_wiring.rs`
