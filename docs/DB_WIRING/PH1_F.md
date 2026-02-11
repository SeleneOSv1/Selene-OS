# PH1.F DB Wiring Spec

## 1) Engine Header

- `engine_id`: `PH1.F`
- `purpose`: Persistence foundation for Selene ledgers/current-state tables with deterministic invariants.
- `version`: `v1`
- `status`: `PASS`

## 2) Data Owned (authoritative)

### `os_core.identities`
- `truth_type`: `CURRENT`
- `primary key`: `user_id`
- invariants:
  - unique `user_id`
  - immutable identity key after insert

### `os_core.devices`
- `truth_type`: `CURRENT`
- `primary key`: `device_id`
- invariants:
  - FK `user_id -> identities.user_id`
  - unique `device_id`

### `os_core.sessions`
- `truth_type`: `CURRENT`
- `primary key`: `session_id`
- invariants:
  - FK `user_id -> identities.user_id`
  - FK `device_id -> devices.device_id`
  - `last_activity_at >= opened_at`

### `memory.memory_ledger`
- `truth_type`: `LEDGER`
- `primary key`: `ledger_id`
- invariants:
  - append-only
  - FK `user_id -> identities.user_id`
  - idempotency dedupe key scope: `(user_id, idempotency_key)`

### `memory.memory_current`
- `truth_type`: `CURRENT`
- `primary key`: `(user_id, memory_key)`
- invariants:
  - rebuildable from `memory_ledger`
  - tombstone rows represented by `active=false`

### `conversation.conversation_ledger`
- `truth_type`: `LEDGER`
- `primary key`: `conversation_turn_id`
- invariants:
  - append-only
  - FK `user_id -> identities.user_id`
  - optional FK `device_id -> devices.device_id`
  - optional FK `session_id -> sessions.session_id`
  - unique `(correlation_id, turn_id)`
  - idempotency dedupe scope: `(correlation_id, idempotency_key)`

### `audit.audit_events`
- `truth_type`: `LEDGER`
- `primary key`: `event_id`
- invariants:
  - append-only
  - reason_code required
  - bounded `payload_min`
  - scoped idempotency dedupe: `(tenant_id, work_order_id, idempotency_key)` when scope exists
  - legacy fallback dedupe: `(correlation_id, idempotency_key)` when scope is absent

## 3) Reads (dependencies)

### identity FK validation
- reads: `identities.user_id`
- keys/joins: direct key lookup
- required indices: PK on `identities.user_id`
- scope rules: user-scoped
- why: enforce FK before inserts into `devices`, `sessions`, `memory_ledger`, `conversation_ledger`

### device/session FK validation
- reads: `devices.device_id`, `sessions.session_id`
- keys/joins: direct key lookup
- required indices: PK on `devices.device_id`, PK on `sessions.session_id`
- scope rules: device/session-scoped
- why: enforce conversation and session integrity

### idempotency dedupe lookups
- reads:
  - `memory_ledger.idempotency_key`
  - `conversation_ledger.idempotency_key`
  - `audit_events.idempotency_key`
- keys/joins: hash-key map lookup by dedupe scope
- required indices:
  - `(user_id, idempotency_key)`
  - `(correlation_id, idempotency_key)`
  - `(tenant_id, work_order_id, idempotency_key)`
- scope rules: per user/per correlation and per `(tenant_id, work_order_id)` when present
- why: deterministic no-op for retries

## 4) Writes (outputs)

### append memory ledger event
- writes:
  - `memory_ledger(ledger_id, user_id, event, idempotency_key)`
  - `memory_current(...)` materialized update
- ledger event_type: `MEMORY_STORED | MEMORY_FORGOTTEN`
- required fields: `user_id`, `memory_key`, `kind`, `t_event`, `reason_code`
- idempotency_key rule (current slice): caller-provided deterministic key, deduped by `(user_id, idempotency_key)`
- failure reason codes: FK violation, duplicate key, contract violation

### append conversation turn
- writes: `conversation_ledger(...)`
- ledger event_type: `CONVERSATION_TURN_STORED`
- required fields: `correlation_id`, `turn_id`, `user_id`, `role`, `source`, `text_hash`
- idempotency_key rule (current slice): caller-provided deterministic key, deduped by `(correlation_id, idempotency_key)`
- failure reason codes: FK violation, append-only violation, contract violation

### append audit event
- writes: `audit_events(...)`
- ledger event_type: from PH1.J canonical event type
- required fields: `engine`, `event_type`, `reason_code`, `severity`, `correlation_id`, `turn_id`
- idempotency_key rule (current slice):
  - scoped dedupe: `(tenant_id, work_order_id, idempotency_key)` when scope exists
  - fallback dedupe: `(correlation_id, idempotency_key)` when scope is absent
- failure reason codes: append-only violation, contract violation

## 5) Relations & Keys

- `devices.user_id -> identities.user_id`
- `sessions.user_id -> identities.user_id`
- `sessions.device_id -> devices.device_id`
- `conversation_ledger.user_id -> identities.user_id`
- `conversation_ledger.device_id -> devices.device_id` (nullable)
- `conversation_ledger.session_id -> sessions.session_id` (nullable)

Unique constraints:
- `identities(user_id)`
- `devices(device_id)`
- `sessions(session_id)`
- `conversation_ledger(correlation_id, turn_id)`
- dedupe:
  - `memory_ledger(user_id, idempotency_key)`
  - `conversation_ledger(correlation_id, idempotency_key)`
  - `audit_events(tenant_id, work_order_id, idempotency_key)` (scoped)
  - `audit_events(correlation_id, idempotency_key)`

State machine constraints:
- ledgers are append-only; no UPDATE/DELETE mutation path
- `memory_current` rebuilt deterministically from `memory_ledger`

## 6) Audit Emissions (PH1.J)

PH1.F does not author business audit semantics; PH1.J owns canonical event schema and event meaning.
PH1.F owns append-only persistence of PH1.J events via `audit_events`.

Required persisted fields for PH1.J events:
- `event_type`
- `reason_code`
- `payload_min` (allowlisted key set)
- `evidence_ref` (reference-only)

## 7) Acceptance Tests (DB Wiring Proof)

- `AT-F-DB-01` tenant isolation enforced
- `AT-F-DB-02` append-only enforcement (ledger no mutation)
- `AT-F-DB-03` idempotency dedupe works
- `AT-F-DB-04` rebuild current from ledger

Implementation references:
- storage: `crates/selene_storage/src/ph1f.rs`
- repo interface: `crates/selene_storage/src/repo.rs`
- tests: `crates/selene_storage/tests/ph1_f/db_wiring.rs`
- verification: `cargo test -p selene_storage`
