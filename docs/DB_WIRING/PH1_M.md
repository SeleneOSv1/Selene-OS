# PH1.M DB Wiring Spec

## 1) Engine Header

- `engine_id`: `PH1.M`
- `purpose`: Persist deterministic, consent-aware user memory with append-only history and rebuildable current state.
- `version`: `v1`
- `status`: `PASS`

## 2) Data Owned (authoritative)

### `memory_ledger`
- `truth_type`: `LEDGER`
- `primary key`: `ledger_id`
- invariants:
  - append-only memory history (`Stored | Updated | Forgotten`)
  - idempotent retries dedupe deterministically per `(user_id, idempotency_key)`
  - every event is contract-validated before commit

### `memory_current`
- `truth_type`: `CURRENT`
- `primary key`: `(user_id, memory_key)`
- invariants:
  - materialized current projection rebuilt from `memory_ledger`
  - forgotten events keep tombstoned inactive current rows (`active=false`, `memory_value=null`)
  - updates are deterministic by `(user_id, memory_key)`

## 3) Reads (dependencies)

- identity prerequisite:
  - writes require `identities.user_id` to exist
- conversation linkage:
  - memory provenance/evidence may reference conversation/session metadata, but PH1.M truth remains in `memory_ledger` + `memory_current`
- isolation boundary:
  - user memory keys are scoped by `user_id`; same `memory_key` across users must not collide

## 4) Writes (outputs)

### `M_STORE_OR_UPDATE_COMMIT`
- writes: append `memory_ledger` + upsert `memory_current`
- idempotency key rule:
  - dedupe by `(user_id, idempotency_key)` when key is provided

### `M_FORGET_COMMIT`
- writes: append `memory_ledger` + tombstone `memory_current` for the key
- idempotency key rule:
  - same `(user_id, idempotency_key)` resolves to deterministic no-op/original ledger row

### `M_REBUILD_CURRENT`
- writes: rebuild `memory_current` projection deterministically from `memory_ledger`
- hard rule:
  - no direct current-table mutations that bypass ledger semantics

## 5) Relations & Keys

- `memory_ledger.user_id` -> `identities.user_id` (FK in migration baseline; enforced in storage runtime)
- `memory_current.user_id` -> `identities.user_id` (FK in migration baseline; enforced by write path)
- no cross-user key collisions: `(user_id, memory_key)` is the current-state identity

## 6) Audit/Proof Emissions

Row 23 lock scope is PH1.M storage correctness on memory tables.
Proof obligations:
- append-only enforcement for `memory_ledger`
- deterministic idempotency dedupe
- deterministic rebuild parity (`memory_current` before/after rebuild)

## 7) Acceptance Tests (DB Wiring Proof)

- `AT-M-DB-01` tenant/user isolation enforced
  - `at_m_db_01_tenant_isolation_enforced`
- `AT-M-DB-02` append-only enforced
  - `at_m_db_02_append_only_enforced`
- `AT-M-DB-03` idempotency dedupe works
  - `at_m_db_03_idempotency_dedupe_works`
- `AT-M-DB-04` rebuild current from ledger
  - `at_m_db_04_rebuild_current_from_ledger`

Implementation references:
- storage wiring: `crates/selene_storage/src/ph1f.rs`
- typed repo: `crates/selene_storage/src/repo.rs`
- tests: `crates/selene_storage/tests/ph1_m/db_wiring.rs`
