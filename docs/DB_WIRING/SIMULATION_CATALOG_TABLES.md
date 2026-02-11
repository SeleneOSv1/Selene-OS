# Simulation Catalog Tables DB Wiring Spec

## 1) Engine Header

- `engine_id`: `SIMULATION_CATALOG_TABLES`
- `purpose`: Lock DB wiring for simulation catalog definitions as append-only ledger + deterministic current projection.
- `version`: `v1`
- `status`: `PASS`

## 2) Data Owned (authoritative)

Target tables in this slice:
- `os_process.simulation_catalog` (`LEDGER`)
  - primary key: `simulation_catalog_event_id`
  - idempotency unique key: `(tenant_id, simulation_id, simulation_version, idempotency_key)` when key is non-null
  - append-only invariant: no update/delete path allowed
- `os_process.simulation_catalog_current` (`CURRENT`)
  - primary key: `(tenant_id, simulation_id)`
  - projection invariant: every row must reference `source_event_id` from `simulation_catalog`

## 3) Reads (dependencies)

Read paths:
- `simulation_catalog` replay ordered by `simulation_catalog_event_id`
- `simulation_catalog_current` lookup by `(tenant_id, simulation_id)`

Scope rules:
- all reads are tenant-scoped
- no cross-tenant read path

Required indices:
- `ux_simulation_catalog_idempotency`
- `ix_simulation_catalog_tenant_simulation_event`
- `ux_simulation_catalog_current_tenant_simulation_version`

## 4) Writes (outputs)

Write paths:
- append `simulation_catalog` via `SimulationCatalogEventInput`
- project each appended row into `simulation_catalog_current` deterministically
- dedupe retried writes by idempotency scope:
  - `(tenant_id, simulation_id, simulation_version, idempotency_key)`

Failure reason classes:
- contract validation failure
- idempotency conflict (returns original event id; no-op)

## 5) Relations & Keys

Key constraints:
- `simulation_catalog.simulation_catalog_event_id` is monotonic append id
- `simulation_catalog_current.source_event_id` FK -> `simulation_catalog.simulation_catalog_event_id`
- `simulation_catalog_current` primary key enforces one current route per `(tenant_id, simulation_id)`

State constraints:
- `simulation_catalog` is append-only
- `simulation_catalog_current` is rebuildable from ordered `simulation_catalog` rows

## 6) Audit Emissions (PH1.J)

This row locks DB wiring for simulation catalog tables. Runtime activation/deprecation audit emission remains through PH1.J with:
- tenant/work_order/correlation scope
- reason-coded governance transitions

## 7) Acceptance Tests (DB Wiring Proof)

- `AT-SIMCAT-DB-01` tenant isolation enforced
  - `at_simcat_db_01_tenant_isolation_enforced`
- `AT-SIMCAT-DB-02` append-only enforcement
  - `at_simcat_db_02_append_only_enforced`
- `AT-SIMCAT-DB-03` idempotency dedupe works
  - `at_simcat_db_03_idempotency_dedupe_works`
- `AT-SIMCAT-DB-04` rebuild current from ledger
  - `at_simcat_db_04_rebuild_current_from_ledger`

Implementation references:
- kernel contracts: `crates/selene_kernel_contracts/src/ph1simcat.rs`
- storage wiring: `crates/selene_storage/src/ph1f.rs`
- migration: `crates/selene_storage/migrations/0004_simulation_catalog_tables.sql`
- typed repo: `crates/selene_storage/src/repo.rs`
- tests: `crates/selene_storage/tests/simulation_catalog/db_wiring.rs`
