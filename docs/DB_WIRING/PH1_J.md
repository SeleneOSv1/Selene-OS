# PH1.J DB Wiring Spec

## Phase A Artifact Trust Proof Closure Alignment (2026-03-10)
- For the Phase A artifact-trust stack, PH1.J consumes canonical A3 trust transport only:
  - ordered `ArtifactTrustDecisionRecord`
  - envelope-carried `artifact_trust_state`
- PH1.J emits canonical per-artifact proof linkage only:
  - `ArtifactTrustProofEntry`
  - per-artifact `proof_entry_ref`
  - turn-level `proof_record_ref`
- `proof_entry_ref` and `proof_record_ref` are not interchangeable.
- Negative verification outcomes are mandatory proof subjects and remain part of the canonical proof payload.
- Runtime closure proof for this surface lives in:
  - `crates/selene_kernel_contracts/src/ph1j.rs`
  - `crates/selene_os/src/ph1j.rs`
  - tests `at_j_runtime_01` through `at_j_runtime_04`

## 1) Engine Header

- `engine_id`: `PH1.J`
- `purpose`: Canonical append-only audit event writer and query surface for deterministic replay/proof.
- `version`: `v1`
- `status`: `PASS`

## 2) Data Owned (authoritative)

### `audit.audit_events`
- `truth_type`: `LEDGER`
- `primary key`: `event_id`
- invariants:
  - append-only
  - `reason_code` required
  - `payload_min` bounded/allowlisted
  - scoped idempotency dedupe by `(tenant_id, work_order_id, idempotency_key)` when scope exists
  - legacy fallback dedupe by `(correlation_id, idempotency_key)` for unscoped legacy events

## 3) Reads (dependencies)

### replay and correlation queries
- reads: `audit_events.*` filtered by `correlation_id`
- keys/joins used: index/filter by `correlation_id`, optional `turn_id`
- required indices:
  - `audit_events(event_id)` PK
  - `audit_events(correlation_id, idempotency_key)` unique (legacy fallback)
  - `audit_events(tenant_id, work_order_id, idempotency_key)` unique (scoped dedupe)
- scope rules:
  - tenant isolation enforced by `tenant_id` filtering
  - work-order thread linkage enforced by `work_order_id` when present
- why this read is required:
  - deterministic replay and proof extraction

## 4) Writes (outputs)

### append audit event
- writes:
  - `audit_events(event_id, created_at, tenant_id, work_order_id, session_id, user_id, device_id, engine, event_type, reason_code, severity, correlation_id, turn_id, payload_min, evidence_ref, idempotency_key)`
- ledger event_type:
  - `GATE_PASS | GATE_FAIL | STATE_TRANSITION | ... | J_REDACT_APPLIED | ...`
- required fields:
  - `engine`, `event_type`, `reason_code`, `severity`, `correlation_id`, `turn_id`, `payload_min`
  - `tenant_id` + `work_order_id` required for scoped enterprise execution traces
- idempotency_key rule (current slice):
  - caller-provided deterministic key
  - scoped dedupe: `(tenant_id, work_order_id, idempotency_key)` when scope exists
  - fallback dedupe: `(correlation_id, idempotency_key)` only when scope is absent
- failure reason codes:
  - contract violation
  - append-only violation on mutation attempts

### artifact trust proof linkage
- protected trust-governed actions append one canonical proof record containing ordered per-artifact proof entries derived from A3 decision-record order
- PH1.J does not create a second trust transport and does not re-verify trust
- raw proof fragments are non-canonical for Phase A artifact-trust closure once `proof_entry_ref` / `proof_record_ref` exist

## 5) Relations & Keys

FKs:
- none enforced at storage layer in current slice

unique constraints:
- `audit_events(event_id)` PK
- `audit_events(correlation_id, idempotency_key)` when key is present (legacy)
- `audit_events(tenant_id, work_order_id, idempotency_key)` when all are present

state machine constraints:
- append-only ledger; no UPDATE/DELETE path

## 6) Audit Emissions (PH1.J)

PH1.J emits canonical audit events into `audit_events` for all engine gates/decisions.

Minimum emitted event types include:
- `GATE_PASS`
- `GATE_FAIL`
- `STATE_TRANSITION`
- `TOOL_FAIL`
- `J_REDACT_APPLIED`

Payload policy:
- `payload_min` allowlisted keys only
- `evidence_ref` must be reference-only (no raw sensitive payloads)

### related engine boundary: `PH1.KMS`
- KMS-originated audit events must include opaque references only (`secret_ref`, `secret_handle`, `ephemeral_credential_ref`).
- Raw secret material must never appear in `payload_min`, `evidence_ref`, or any PH1.J persisted field.

### related engine boundary: `PH1.EXPORT`
- Export completion events must include bounded metadata only (`export_artifact_id`, `export_hash`, `export_payload_ref`, `export_scope_ref`).
- `export_hash` must be present for tamper-evident replay proofs.
- Raw audio references are forbidden by default; any violation must fail closed before append.

### related engine boundary: `PH1.EXPLAIN`
- Explain-related audit context must remain reason-coded and bounded (`primary_reason_code`, optional related reason list, optional `verbatim_trigger` hash/reference).
- PH1.J must never persist provider internals, threshold values, or chain-of-thought fields for PH1.EXPLAIN consumption paths.

## 7) Acceptance Tests (DB Wiring Proof)

Required by design lock:
- `AT-J-DB-01` tenant isolation enforced
- `AT-J-DB-02` append-only enforcement
- `AT-J-DB-03` idempotency dedupe works
- `AT-J-DB-04` rebuild current from ledger (N/A - ledger-only)

Implemented test coverage:
- `AT-J-DB-01` `at_j_db_01_tenant_isolation_enforced`
- `AT-J-DB-02` `at_j_db_02_append_only_enforced`
- `AT-J-DB-03` `at_j_db_03_idempotency_dedupe_works`
- `AT-J-DB-04` `at_j_db_04_ledger_only_no_current_rebuild_required`
- artifact-trust runtime proof coverage:
  - `at_j_runtime_01_protected_action_writes_canonical_proof_record`
  - `at_j_runtime_02_forced_failure_is_structured`
  - `at_j_runtime_03_artifact_trust_entries_follow_decision_order`
  - `at_j_runtime_04_receipt_updates_artifact_trust_state_with_proof_linkage`

Code references:
- contract: `crates/selene_kernel_contracts/src/ph1j.rs`
- storage: `crates/selene_storage/src/ph1f.rs`
- repo interface: `crates/selene_storage/src/repo.rs`
- tests: `crates/selene_storage/tests/ph1_j/db_wiring.rs`
