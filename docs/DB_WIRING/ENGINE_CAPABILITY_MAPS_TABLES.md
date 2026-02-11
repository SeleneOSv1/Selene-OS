# Engine Capability Maps Tables DB Wiring Spec

## 1) Engine Header

- `engine_id`: `ENGINE_CAPABILITY_MAPS_TABLES`
- `purpose`: Lock DB wiring for engine capability maps as append-only ledger + deterministic current projection.
- `version`: `v1`
- `status`: `PASS`

## 2) Data Owned (authoritative)

Target tables in this slice:
- `os_process.engine_capability_maps` (`LEDGER`)
  - primary key: `engine_capability_map_event_id`
  - idempotency unique key: `(tenant_id, engine_id, capability_id, capability_map_version, idempotency_key)` when key is non-null
  - append-only invariant: no update/delete path allowed
- `os_process.engine_capability_maps_current` (`CURRENT`)
  - primary key: `(tenant_id, engine_id, capability_id)`
  - projection invariant: every row must reference `source_event_id` from `engine_capability_maps`

## 3) Reads (dependencies)

Read paths:
- `engine_capability_maps` replay ordered by `engine_capability_map_event_id`
- `engine_capability_maps_current` lookup by `(tenant_id, engine_id, capability_id)`

Scope rules:
- all reads are tenant-scoped
- no cross-tenant read path

Required indices:
- `ux_engine_capability_maps_idempotency`
- `ix_engine_capability_maps_tenant_engine_cap_event`
- `ux_engine_capability_maps_current_tenant_engine_cap_version`

## 4) Writes (outputs)

Write paths:
- append `engine_capability_maps` via `EngineCapabilityMapEventInput`
- project each appended row into `engine_capability_maps_current` deterministically
- dedupe retried writes by idempotency scope:
  - `(tenant_id, engine_id, capability_id, capability_map_version, idempotency_key)`

Failure reason classes:
- contract validation failure
- idempotency conflict (returns original event id; no-op)

## 5) Relations & Keys

Key constraints:
- `engine_capability_maps.engine_capability_map_event_id` is monotonic append id
- `engine_capability_maps_current.source_event_id` FK -> `engine_capability_maps.engine_capability_map_event_id`
- `engine_capability_maps_current` primary key enforces one current route per `(tenant_id, engine_id, capability_id)`

State constraints:
- `engine_capability_maps` is append-only
- `engine_capability_maps_current` is rebuildable from ordered `engine_capability_maps` rows

## 6) Audit Emissions (PH1.J)

This row locks DB wiring for engine capability maps tables. Runtime activation/deprecation audit emission remains through PH1.J with:
- tenant/work_order/correlation scope
- reason-coded governance transitions

## 7) Acceptance Tests (DB Wiring Proof)

- `AT-ECM-DB-01` tenant isolation enforced
  - `at_ecm_db_01_tenant_isolation_enforced`
- `AT-ECM-DB-02` append-only enforcement
  - `at_ecm_db_02_append_only_enforced`
- `AT-ECM-DB-03` idempotency dedupe works
  - `at_ecm_db_03_idempotency_dedupe_works`
- `AT-ECM-DB-04` rebuild current from ledger
  - `at_ecm_db_04_rebuild_current_from_ledger`

Implementation references:
- kernel contracts: `crates/selene_kernel_contracts/src/ph1ecm.rs`
- storage wiring: `crates/selene_storage/src/ph1f.rs`
- migration: `crates/selene_storage/migrations/0005_engine_capability_maps_tables.sql`
- typed repo: `crates/selene_storage/src/repo.rs`
- tests: `crates/selene_storage/tests/engine_capability_maps/db_wiring.rs`
