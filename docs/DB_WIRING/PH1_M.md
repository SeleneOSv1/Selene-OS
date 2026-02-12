# PH1.M DB Wiring Spec (vNext)

Contract scope note:
- This file is schema/wiring contract only.
- Behavioral memory narrative is canonical in `docs/12_MEMORY_ARCHITECTURE.md`.

## 1) Engine Header

- `engine_id`: `PH1.M`
- `purpose`: Persist memory atoms/retrieval controls and emotional continuity state for deterministic continuity only (never authority).
- `version`: `vNext`
- `status`: `DONE (design-level)`

## 2) Ownership (Mandatory Lock)

### PH1.M owns (domain semantics + write authority)
- memory atoms (`memory_atoms_ledger`, `memory_atoms_current`)
- suppression controls (`memory_suppression_rules` for `DO_NOT_MENTION | DO_NOT_REPEAT | DO_NOT_STORE`)
- emotional continuity state (`emotional_threads_ledger`, `emotional_threads_current`)
- thread continuity state (`memory_threads_ledger`, `memory_threads_current`, `memory_thread_refs`)
- retrieval index state (`memory_graph_nodes`, `memory_graph_edges`)
- archive pointer index (`memory_archive_index`, pointer-only placeholder)
- memory quality telemetry (`memory_metrics_ledger`)

### PH1.F owns (storage spine)
- physical Postgres schemas/tables
- migrations
- key/constraint enforcement
- append-only ledger and rebuildable-current invariants

### PH1.J owns (audit schema)
- audit envelope schema and append-only proof trail
- reason-code contract shape and audit field requirements

### Engine B owns (canonical)
- Device Vault + Outbox mechanics: crash durability, replay, ack-gated deletion, idempotent sync
- canonical path: External canonical (Rust-core repo): `crates/rust_core/docs/engines/B_DEVICE_VAULT_OUTBOX.md`
- PH1.M emits memory deltas to Engine B outbox interfaces; PH1.M does not own vault/outbox mechanics

### PH1.LEARN / PH1.KNOW owns
- artifact packaging and vocabulary-pack generation
- PH1.M consumes packaged artifacts as hints only

### PH1.M must never
- grant authority
- execute actions
- bypass Access/simulation gates
- override authoritative current-state truth
- dump raw ledger/history as user output

## 3) Data Owned (Authoritative, Design-Level)

### `memory.memory_atoms_ledger`
- `truth_type`: `LEDGER`
- `primary key`: `memory_atom_event_id`
- invariants:
  - append-only (no update/delete)
  - idempotent dedupe on `(tenant_id, user_id, idempotency_key)`
  - reason-coded event types only (`ATOM_STORED | ATOM_UPDATED | ATOM_FORGOTTEN`)

### `memory.memory_atoms_current`
- `truth_type`: `CURRENT`
- `primary key`: `(tenant_id, user_id, atom_key)`
- invariants:
  - deterministic rebuild from `memory_atoms_ledger`
  - tombstone/unresolved state is explicit (no silent deletion)

### `memory.memory_suppression_rules`
- `truth_type`: `CURRENT (CONFIG)`
- `primary key`: `(tenant_id, user_id, target_type, target_id, rule_kind)`
- invariants:
  - `rule_kind` in `DO_NOT_MENTION | DO_NOT_REPEAT | DO_NOT_STORE`
  - `target_type` in `THREAD_ID | WORK_ORDER_ID | TOPIC_KEY`
  - `target_id` is a stable identifier for the selected `target_type`
  - deterministic precedence by `updated_at`
  - user-scoped only (no cross-user effects)
  - required metadata: `created_at`, `reason_code`

### `memory.emotional_threads_ledger`
- `truth_type`: `LEDGER`
- `primary key`: `emotional_thread_event_id`
- invariants:
  - append-only
  - tone-continuity payload only (no authority/fact mutation fields)

### `memory.emotional_threads_current`
- `truth_type`: `CURRENT`
- `primary key`: `(tenant_id, user_id, thread_key)`
- invariants:
  - rebuildable from `emotional_threads_ledger`
  - exposure gating fields must be enforced before composition

### `memory.memory_metrics_ledger`
- `truth_type`: `LEDGER`
- `primary key`: `memory_metric_event_id`
- invariants:
  - append-only
  - non-authoritative metrics only
  - bounded `payload_min` keys only

### `memory.memory_threads_ledger`
- `truth_type`: `LEDGER`
- `primary key`: `memory_thread_event_id`
- invariants:
  - append-only
  - deterministic event types only (`THREAD_DIGEST_UPSERT | THREAD_RESOLVED | THREAD_FORGOTTEN`)
  - idempotent dedupe on `(tenant_id, user_id, idempotency_key)`

### `memory.memory_threads_current`
- `truth_type`: `CURRENT`
- `primary key`: `(tenant_id, user_id, thread_id)`
- invariants:
  - rebuildable from `memory_threads_ledger`
  - bounded digest fields (`thread_title`, `key_entities`, `last_decisions`, `unresolved_questions`, `next_step`)
  - continuity policy fields:
    - `pinned` (bool)
    - `unresolved` (bool)
    - `unresolved_deadline_at` (timestamp; default 90-day unresolved limit)
    - `last_used_at` (timestamp)
    - `last_updated_at` (timestamp)
  - retention tier is derived deterministically from timestamps/policy (`HOT | WARM | COLD`), not stored as mutable truth

### `memory.memory_thread_refs`
- `truth_type`: `CURRENT (POINTER_SET)`
- `primary key`: `(tenant_id, user_id, thread_id, conversation_turn_id)`
- invariants:
  - stores references only (no raw transcript text)
  - bounded pointer count per thread per policy

### `memory.memory_graph_nodes`
- `truth_type`: `CURRENT (INDEX)`
- `primary key`: `(tenant_id, user_id, node_id)`
- invariants:
  - non-authoritative retrieval index only
  - node kinds bounded to `ENTITY | PROJECT | VENDOR | DECISION | THREAD`

### `memory.memory_graph_edges`
- `truth_type`: `CURRENT (INDEX)`
- `primary key`: `(tenant_id, user_id, edge_id)`
- invariants:
  - non-authoritative retrieval index only
  - edge kinds bounded to `MENTIONED_WITH | DEPENDS_ON | DECIDED_IN | BLOCKED_BY`
  - deterministic uniqueness on `(tenant_id, user_id, from_node_id, to_node_id, edge_kind)`

### `memory.memory_archive_index` (optional placeholder)
- `truth_type`: `CURRENT (INDEX POINTERS)`
- `primary key`: `(tenant_id, user_id, archive_ref_id)`
- invariants:
  - pointer/index metadata only (no forced vector storage decision here)
  - every row must reference `conversation_ledger` evidence pointers
  - rebuildable from atoms + thread digests + archive pointers

## 4) Reads (Dependencies)

### Required reads
- `memory_atoms_current` by `(tenant_id, user_id)` for context/hint composition
- `memory_suppression_rules` by `(tenant_id, user_id)` for exposure gating
- `emotional_threads_current` by `(tenant_id, user_id)` for tone continuity hints
- `memory_threads_current` by `(tenant_id, user_id)` for resume bundle selection
- `memory_thread_refs` by `(tenant_id, user_id, thread_id)` for bounded evidence pointers
- `memory_graph_nodes`/`memory_graph_edges` by `(tenant_id, user_id)` for deterministic ranked retrieval
- `memory_archive_index` by `(tenant_id, user_id)` for bounded page-in candidate selection
- pending WorkOrder selectors from `os_core.work_orders_current` for continuity offers (`status in DRAFT|CLARIFY|CONFIRM`)
- bounded evidence refs from `conversation_ledger`/`audit_events` when generating safe recall summaries

### Required indices
- `memory_atoms_current(tenant_id, user_id, atom_key)`
- `memory_suppression_rules(tenant_id, user_id, rule_key)`
- `emotional_threads_current(tenant_id, user_id, thread_key)`
- `memory_threads_current(tenant_id, user_id, thread_id)`
- `memory_thread_refs(tenant_id, user_id, thread_id, conversation_turn_id)`
- `memory_graph_nodes(tenant_id, user_id, node_id)`
- `memory_graph_edges(tenant_id, user_id, from_node_id, to_node_id, edge_kind)`
- `memory_archive_index(tenant_id, user_id, archive_ref_id)`
- idempotency keys on all PH1.M ledgers

### Scope rules
- tenant isolation mandatory
- user isolation mandatory
- no cross-tenant or cross-user personalization reads

## 5) Writes (Outputs)

All PH1.M retriable writes use deterministic idempotency:
- `idempotency_key = hash(tenant_id + work_order_id + step_id + step_input_hash)`
- if `work_order_id` is not present, caller policy must provide a deterministic substitute business key in `step_input_hash` scope

### Atom writes
- append `memory_atoms_ledger`
- project into `memory_atoms_current`

### Suppression writes
- upsert `memory_suppression_rules`
- supports target-specific suppression on `THREAD_ID | WORK_ORDER_ID | TOPIC_KEY`

### Emotional writes
- append `emotional_threads_ledger`
- project into `emotional_threads_current`

### Thread writes
- on session soft-close/close, append thread digest event to `memory_threads_ledger`
- project current thread state into `memory_threads_current`
- upsert bounded evidence pointers into `memory_thread_refs`

### Graph/index writes
- upsert bounded retrieval nodes/edges in `memory_graph_nodes` and `memory_graph_edges`
- maintain optional archive page-in pointers in `memory_archive_index`

### Metrics writes
- append `memory_metrics_ledger` (bounded telemetry only)

### Context build paging behavior
- context/hint composition prefers thread digest summary first
- then graph-ranked entities
- then up to 2 archive excerpts by pointer
- hard caps remain enforced (`<=32KB`, `<=20 atoms`, `<=2 excerpts`)
- if `DO_NOT_MENTION` suppression matches selected `thread_id` or `work_order_id`, those items must not surface

### Engine B handoff (non-owned)
- emit memory delta envelopes for Engine B outbox delivery
- PH1.M does not implement outbox ack/replay semantics

## 6) Relations & Keys

Logical FK expectations (enforced by PH1.F contracts):
- all `memory.*.tenant_id` rows are tenant-scoped
- all `memory.*.user_id` rows resolve to identity scope

Unique constraints:
- current tables unique on `(tenant_id, user_id, domain_key)`
- ledgers unique on idempotency scopes
- `memory_thread_refs` unique on `(tenant_id, user_id, thread_id, conversation_turn_id)`
- `memory_graph_edges` unique on `(tenant_id, user_id, from_node_id, to_node_id, edge_kind)`

State constraints:
- all PH1.M ledgers are append-only
- all PH1.M current tables are rebuildable from ledgers

## 7) Audit Emissions (PH1.J)

PH1.M emits reason-coded audit events (schema owned by PH1.J):
- `MEM_ATOM_STORED`
- `MEM_ATOM_UPDATED`
- `MEM_ATOM_FORGOTTEN`
- `MEM_SUPPRESSION_RULE_SET`
- `MEM_EMO_THREAD_UPDATED`
- `MEM_SAFE_SUMMARY_QUERIED`
- `MEM_METRICS_EMITTED`
- `MEM_CLARIFY_REQUIRED`

Payload discipline:
- bounded `payload_min` keys only
- evidence references only (no raw transcript dumps)

## 8) Acceptance Tests (Design IDs)

- `AT-MEM-DB-01` tenant isolation enforced
- `AT-MEM-DB-02` append-only enforced for all PH1.M ledgers
- `AT-MEM-DB-03` deterministic idempotency dedupe
- `AT-MEM-DB-04` current rebuild parity from ledgers
- `AT-MEM-DB-05` suppression enforcement prevents leakage
- `AT-MEM-DB-06` emotional-thread writes remain tone-only
- `AT-MEM-THREAD-01` Thread digest persists on close with pointer refs only (no raw dumps)
- `AT-MEM-GRAPH-01` graph retrieval remains bounded with deterministic ordering
- `AT-MEM-PAGE-01` paging path enforces hard caps for atoms/excerpts/bytes
- `AT-MEM-RESUME-01` auto-resume selects recent eligible thread only when identity is OK
- `AT-MEM-PREF-01` `memory_retention_mode` (`DEFAULT | REMEMBER_EVERYTHING`) changes retention/resume policy deterministically
- `AT-MEM-RESUME-02` `HOT/WARM/COLD` behavior is deterministic and bounded (`72h/30d/forever`)
- `AT-MEM-THREAD-02` pinned/unresolved retention rules enforced (including 90-day unresolved decay)
- `AT-MEM-SUPPRESS-01` `DO_NOT_MENTION` may target `work_order_id` and suppress pending-work resume suggestions

## 10) Blocker

- `none`
