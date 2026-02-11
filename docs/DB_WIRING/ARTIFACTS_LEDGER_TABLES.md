# Artifacts Ledger + Tool Cache DB Wiring Spec

## 1) Engine Header

- `engine_id`: `ARTIFACTS_LEDGER_TABLES`
- `purpose`: Lock DB wiring for `artifacts_ledger` (append-only) and optional `tool_cache` (read-only helper cache).
- `version`: `v1`
- `status`: `PASS`

## 2) Data Owned (authoritative)

Target tables in this slice:
- `os_process.artifacts_ledger` (`LEDGER`)
  - primary key: `artifact_id`
  - unique key: `(scope_type, scope_id, artifact_type, artifact_version)`
  - idempotency unique key: `(scope_type, scope_id, artifact_type, artifact_version, idempotency_key)` when key is non-null
  - append-only invariant: no update/delete path allowed
- `os_core.tool_cache` (`CACHE`, optional)
  - primary key: `cache_id`
  - unique key: `(tool_name, query_hash, locale)` (deterministic upsert key)
  - TTL invariant: `expires_at` governs read eligibility

## 3) Reads (dependencies)

Read paths:
- `artifacts_ledger` ordered by `artifact_id` (replay/audit chronology)
- `artifacts_ledger` lookup by `(scope_type, scope_id, artifact_type, artifact_version)`
- `tool_cache` lookup by `(tool_name, query_hash, locale)` with `expires_at > now`

Scope rules:
- artifact reads are scope-bounded by `(scope_type, scope_id)`
- tenant isolation is enforced via scope partitioning (`scope_type=TENANT`, `scope_id=<tenant_id>`)
- cache lookups are deterministic by tool/query/locale key

Required indices:
- `ux_artifacts_ledger_scope_type_scope_id_type_version`
- `ux_artifacts_ledger_idempotency`
- `ix_artifacts_ledger_scope_type_scope_id_type_artifact_id`
- `ux_tool_cache_tool_query_locale`
- `ix_tool_cache_expires_at`

## 4) Writes (outputs)

Write paths:
- append `artifacts_ledger` via `ArtifactLedgerRowInput`
- dedupe retried writes by idempotency scope:
  - `(scope_type, scope_id, artifact_type, artifact_version, idempotency_key)`
- upsert `tool_cache` by deterministic cache key:
  - `(tool_name, query_hash, locale)`; same key updates payload + expiry in place

Failure reason classes:
- contract validation failure
- duplicate scope+version conflict in artifact ledger
- idempotency conflict (returns original `artifact_id`; no-op)

## 5) Relations & Keys

Key constraints:
- `artifacts_ledger.artifact_id` is monotonic append id
- `artifacts_ledger` enforces one row per `(scope_type, scope_id, artifact_type, artifact_version)`
- `tool_cache` enforces one current cached row per `(tool_name, query_hash, locale)`

State constraints:
- `artifacts_ledger` is append-only
- rollback/deprecation is expressed by new ledger rows (`status`), never in-place mutation
- `tool_cache` is TTL-bound and non-authoritative

## 6) Audit Emissions (PH1.J)

This row locks DB wiring for artifact/cache persistence. Runtime apply/rollback and cache-path audit emission remains through PH1.J with:
- correlation-scoped events
- reason-coded artifact lifecycle transitions
- bounded payload keys

## 7) Acceptance Tests (DB Wiring Proof)

- `AT-ART-DB-01` tenant isolation enforced
  - `at_art_db_01_tenant_isolation_enforced`
- `AT-ART-DB-02` append-only enforcement
  - `at_art_db_02_append_only_enforced`
- `AT-ART-DB-03` idempotency dedupe works
  - `at_art_db_03_idempotency_dedupe_works`
- `AT-ART-DB-04` ledger-only proof (no current projection table)
  - `at_art_db_04_ledger_only_no_current_rebuild_required`
- optional cache proof
  - `at_art_db_05_tool_cache_upsert_and_ttl_read`

Implementation references:
- kernel contracts: `crates/selene_kernel_contracts/src/ph1art.rs`
- storage wiring: `crates/selene_storage/src/ph1f.rs`
- migration: `crates/selene_storage/migrations/0006_artifacts_ledger_and_tool_cache.sql`
- typed repo: `crates/selene_storage/src/repo.rs`
- tests: `crates/selene_storage/tests/artifacts_ledger/db_wiring.rs`
