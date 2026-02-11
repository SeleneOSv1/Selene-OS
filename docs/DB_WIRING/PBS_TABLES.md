# PBS Tables DB Wiring Spec

## 1) Engine Header

- `engine_id`: `PBS_TABLES`
- `purpose`: Lock DB wiring for `blueprint_registry` + `process_blueprints` as the authoritative Process Blueprint System store.
- `version`: `v1`
- `status`: `PASS`

## 2) Data Owned (authoritative)

Target tables in this slice:
- `os_process.process_blueprints` (`LEDGER`)
  - primary key: `process_blueprint_event_id`
  - idempotency unique key: `(tenant_id, process_id, blueprint_version, idempotency_key)` when key is non-null
  - append-only invariant: no update/delete path allowed
- `os_process.blueprint_registry` (`CURRENT/REGISTRY`)
  - primary key: `(tenant_id, intent_type)`
  - one current route per `tenant + intent_type`
  - projection invariant: every row must reference `source_event_id` from `process_blueprints`

## 3) Reads (dependencies)

Read paths:
- `process_blueprints` replay ordered by `process_blueprint_event_id`
- `blueprint_registry` lookup by `(tenant_id, intent_type)`

Scope rules:
- all reads are tenant-scoped
- no cross-tenant read path

Required indices:
- `ux_process_blueprints_idempotency`
- `ix_process_blueprints_tenant_intent_event`
- `ux_blueprint_registry_tenant_process_version`

## 4) Writes (outputs)

Write paths:
- append `process_blueprints` via `ProcessBlueprintEventInput`
- project each appended event into `blueprint_registry` deterministically
- dedupe retried writes by idempotency key scope:
  - `hash_scope = (tenant_id, process_id, blueprint_version, idempotency_key)`

Failure reason classes:
- contract validation failure
- idempotency conflict (returns original event id; no-op)

## 5) Relations & Keys

Key constraints:
- `process_blueprints.process_blueprint_event_id` is monotonic append id
- `blueprint_registry.source_event_id` FK -> `process_blueprints.process_blueprint_event_id`
- `blueprint_registry` primary key enforces one current route per `(tenant_id, intent_type)`

State constraints:
- `process_blueprints` is append-only
- `blueprint_registry` is rebuildable from ordered `process_blueprints` events

## 6) Audit Emissions (PH1.J)

This row locks DB wiring for PBS tables. Runtime audit emission remains through PH1.J with:
- tenant/work_order/correlation scope
- reason-coded governance transitions

## 7) Acceptance Tests (DB Wiring Proof)

- `AT-PBS-DB-01` tenant isolation enforced
  - `at_pbs_db_01_tenant_isolation_enforced`
- `AT-PBS-DB-02` append-only enforcement
  - `at_pbs_db_02_append_only_enforced`
- `AT-PBS-DB-03` idempotency dedupe works
  - `at_pbs_db_03_idempotency_dedupe_works`
- `AT-PBS-DB-04` rebuild current from ledger
  - `at_pbs_db_04_rebuild_current_from_ledger`

Implementation references:
- kernel contracts: `crates/selene_kernel_contracts/src/ph1pbs.rs`
- storage wiring: `crates/selene_storage/src/ph1f.rs`
- migration: `crates/selene_storage/migrations/0003_pbs_tables.sql`
- typed repo: `crates/selene_storage/src/repo.rs`
- tests: `crates/selene_storage/tests/pbs/db_wiring.rs`
