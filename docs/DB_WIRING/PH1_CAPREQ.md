# PH1.CAPREQ DB Wiring Spec

## 1) Engine Header

- `engine_id`: `PH1.CAPREQ`
- `purpose`: Persist deterministic tenant-scoped capability request lifecycle (`Draft | PendingApproval | Approved | Rejected | Fulfilled | Canceled`) with append-only history.
- `version`: `v1`
- `status`: `PASS`

## 2) Data Owned (authoritative)

### `capreq_ledger`
- `truth_type`: `LEDGER`
- `primary key`: `capreq_event_id`
- invariants:
  - append-only lifecycle history per `(tenant_id, capreq_id)`
  - every event is reason-coded and idempotent when retried
  - no in-place mutation; corrections are new events

### `capreq_current`
- `truth_type`: `CURRENT`
- `primary key`: `(tenant_id, capreq_id)`
- invariants:
  - one current projection row per tenant-scoped capability request
  - rebuildable from `capreq_ledger` in deterministic event order
  - source event reference (`source_event_id`) points to latest applied ledger row

## 3) Reads (dependencies)

- identity prerequisite:
  - requester must exist in `identities` before CAPREQ ledger writes
- tenant isolation:
  - all reads/writes are keyed by `(tenant_id, capreq_id)`; cross-tenant access is isolated

## 4) Writes (outputs)

### `CAPREQ_CREATE_DRAFT` (draft)
- writes: append `capreq_ledger`, update `capreq_current`
- idempotency key rule:
  - dedupe by `(tenant_id, capreq_id, idempotency_key)`

### `CAPREQ_SUBMIT_COMMIT` (commit)
- writes: append `capreq_ledger`, update `capreq_current`
- idempotency key rule:
  - dedupe by `(tenant_id, capreq_id, idempotency_key)`

### `CAPREQ_APPROVE_COMMIT` (commit)
- writes: append `capreq_ledger`, update `capreq_current`
- idempotency key rule:
  - dedupe by `(tenant_id, capreq_id, idempotency_key)`

### `CAPREQ_REJECT_COMMIT` (commit)
- writes: append `capreq_ledger`, update `capreq_current`
- idempotency key rule:
  - dedupe by `(tenant_id, capreq_id, idempotency_key)`

## 5) Relations & Keys

- `capreq_current.source_event_id` references `capreq_ledger.capreq_event_id`.
- idempotency uniqueness is tenant-scoped and capreq-scoped.

## 6) Audit/Proof Emissions

Row 26 lock centers on CAPREQ current + lifecycle persistence.
Append-only evidence is provided by `capreq_ledger`; overwrite attempts must fail deterministically.

## 7) Acceptance Tests (DB Wiring Proof)

- `AT-CAPREQ-DB-01` tenant isolation enforced
  - `at_capreq_db_01_tenant_isolation_enforced`
- `AT-CAPREQ-DB-02` append-only enforced
  - `at_capreq_db_02_append_only_enforced`
- `AT-CAPREQ-DB-03` idempotency dedupe works
  - `at_capreq_db_03_idempotency_dedupe_works`
- `AT-CAPREQ-DB-04` rebuild current from ledger
  - `at_capreq_db_04_rebuild_current_from_ledger`

Implementation references:
- storage wiring: `crates/selene_storage/src/ph1f.rs`
- typed repo: `crates/selene_storage/src/repo.rs`
- tests: `crates/selene_storage/tests/ph1_capreq/db_wiring.rs`
