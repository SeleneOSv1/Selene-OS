# PH1.CAPREQ / PH1.CAPREQ.001 DB Wiring Spec

## 1) Engine Header

- `engine_id`: `PH1.CAPREQ`
- `implementation_id`: `PH1.CAPREQ.001`
- `purpose`: Persist deterministic tenant-scoped capability-request lifecycle truth with append-only history and rebuildable current projection.
- `version`: `v1`
- `status`: `PASS`

## 1A) Family Namespace Lock (Row 40)

- `PH1.CAPREQ` is the family namespace and must route only to active implementations.
- active implementation ids (locked):
  - `PH1.CAPREQ.001`
- family dispatch is fail-closed:
  - unknown `implementation_id` -> reject with deterministic contract violation (`ph1capreq.implementation_id`)
  - no implicit fallback to unknown or draft implementation ids

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
  - one current projection row per tenant-scoped request
  - rebuildable from `capreq_ledger` in deterministic event order
  - `source_event_id` points to latest applied ledger row

## 2A) Governance Boundary (Locked)

PH1.CAPREQ records request lifecycle truth; PH1.CAPREQ.001 does not grant authority.

Deterministic boundary rules:
- PH1.CAPREQ.001 persists request state (`DRAFT -> PENDING_APPROVAL -> APPROVED|REJECTED -> FULFILLED|CANCELED`) only.
- Authority remains PH1.ACCESS.001 -> PH2.ACCESS.002.
- All CAPREQ side effects are simulation-gated (`No Simulation -> No Execution`).
- Selene OS orchestrates CAPREQ + ACCESS sequencing; engines do not call engines directly.
- No CAPREQ lifecycle write may be interpreted as execution approval unless Access/approval gates pass in the same orchestrated flow.

## 3) Reads (dependencies)

- identity prerequisite:
  - requester must exist in `identities` before CAPREQ ledger writes
- tenant isolation:
  - all reads/writes are keyed by `(tenant_id, capreq_id)`; cross-tenant access is isolated
- simulation gate prerequisite:
  - PH1.CAPREQ.001 request envelope simulation id/type must match request variant before write path

## 4) Writes (outputs)

### `CAPREQ_CREATE_DRAFT` (draft)
- writes: append `capreq_ledger`, update `capreq_current`
- idempotency key rule:
  - dedupe by `(tenant_id, capreq_id, idempotency_key)`

### `CAPREQ_SUBMIT_FOR_APPROVAL_COMMIT` (commit)
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

### `CAPREQ_FULFILL_COMMIT` (commit)
- writes: append `capreq_ledger`, update `capreq_current`
- idempotency key rule:
  - dedupe by `(tenant_id, capreq_id, idempotency_key)`

### `CAPREQ_CANCEL_REVOKE` (revoke)
- writes: append `capreq_ledger`, update `capreq_current`
- idempotency key rule:
  - dedupe by `(tenant_id, capreq_id, idempotency_key)`

## 5) Relations & Keys

- `capreq_current.source_event_id` references `capreq_ledger.capreq_event_id`.
- idempotency uniqueness is tenant-scoped and capreq-scoped.

## 6) Reason Codes (minimum runtime set)

- `CAPREQ_CREATED`
- `CAPREQ_SUBMITTED`
- `CAPREQ_APPROVED`
- `CAPREQ_REJECTED`
- `CAPREQ_FULFILLED`
- `CAPREQ_CANCELED`

## 7) Audit/Proof Emissions

- Row 26 lock centers on CAPREQ ledger + projection persistence.
- Append-only evidence is provided by `capreq_ledger`; overwrite attempts fail closed.
- Every transition remains reason-coded and replayable by `(tenant_id, capreq_id, capreq_event_id)`.

## 8) Acceptance Tests

Runtime implementation acceptance:
- `AT-CAPREQ-01`: tenant isolation for same `capreq_id` across tenants
- `AT-CAPREQ-02`: append-only transition enforcement (invalid sequence fails closed)
- `AT-CAPREQ-03`: idempotent retries produce deterministic reused/no-op decision payload
- `AT-CAPREQ-04`: current state progression is deterministic from lifecycle sequence
- `AT-CAPREQ-FAMILY-01`: family dispatch rejects unknown implementation ids
- `AT-CAPREQ-FAMILY-02`: active implementation list remains locked to approved ids

DB wiring acceptance:
- `AT-CAPREQ-DB-01` tenant isolation enforced
  - `at_capreq_db_01_tenant_isolation_enforced`
- `AT-CAPREQ-DB-02` append-only enforced
  - `at_capreq_db_02_append_only_enforced`
- `AT-CAPREQ-DB-03` idempotency dedupe works
  - `at_capreq_db_03_idempotency_dedupe_works`
- `AT-CAPREQ-DB-04` rebuild current from ledger
  - `at_capreq_db_04_rebuild_current_from_ledger`

Implementation references:
- kernel contracts: `crates/selene_kernel_contracts/src/ph1capreq.rs`
- engine runtime: `crates/selene_engines/src/ph1capreq.rs`
- os runtime + storage wiring: `crates/selene_os/src/ph1capreq.rs`
- typed repo: `crates/selene_storage/src/repo.rs`
- storage integration tests: `crates/selene_storage/tests/ph1_capreq/db_wiring.rs`
