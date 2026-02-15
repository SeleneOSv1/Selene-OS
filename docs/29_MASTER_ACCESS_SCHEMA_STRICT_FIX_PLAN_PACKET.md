# PH1.ACCESS Master Access Schema Strict Fix Plan Packet (v1)

Last updated: 2026-02-15
Owner: Selene core design + runtime
Status: STEP2_COMPLETED_PENDING_STEP3

## 1) Purpose

This packet upgrades PH1.ACCESS from "per-user gate + override only" to a full schema-driven Master Access system:

- global AP template library (Selene-owned)
- tenant AP schema library (tenant-owned)
- tenant overlay layer
- position AP binding
- per-user compiled access instance lineage
- deterministic escalation with multi-approval and board voting

No existing governance laws are relaxed.

## 2) Non-Negotiable Laws (frozen)

1. Deny-by-default: if not explicitly allowed, it is denied.
2. Deterministic output: same inputs -> same decision.
3. Output contract remains exactly: `ALLOW | DENY | ESCALATE`.
4. No Simulation -> No Execution for every policy, override, approval, and activation action.
5. No silent changes: edits are versioned, reason-coded, audited.
6. Access decides "what can be done"; onboarding decides "what must be collected."
7. Delete is retire (`RETIRED`), never hard delete.

## 3) Core Objects and Relationships

### 3.1 Master Access Profiles (APs)

- Global Master AP (Selene Inc shipped defaults)
- Tenant Master AP (tenant-scoped derived/customized AP schema)

Each AP schema version contains:

- allow list (capabilities/actions)
- constraints (scope, limits, sensitivity)
- escalation hooks (required approvals / board policy)
- ownership/edit authority metadata

Lifecycle:

- `DRAFT -> ACTIVE -> RETIRED`
- edits create new version (no in-place mutation of active versions)

### 3.2 Tenant Overlay

Overlay applies on top of tenant AP (or directly on global AP if tenant has no custom AP version).

Allowed overlay operations:

- add permission (bounded by governance)
- remove permission
- tighten constraints
- add tenant escalation policy hooks

### 3.3 Position Binding

Position binds to:

- one AP baseline
- optional overlay
- optional position-local bounded rules

Position activation must fail closed if AP binding is unresolved or non-active.

### 3.4 Per-User Access Instance

Compiled from:

`global AP -> tenant AP version -> tenant overlay -> position binding -> per-user overrides`

This compiled row is the runtime gate input and must store lineage references for replay.

### 3.5 Per-User Overrides

Override types:

- `ONE_SHOT`
- `TEMPORARY_UNTIL`
- `TIME_WINDOW`
- `PERMANENT_UNTIL_REVOKE`

Overrides are simulation-gated, tenant-scoped, audited, and never mutate global AP.

## 4) Ownership Map

### 4.1 Selene Inc owns

- global AP template library
- global governance limits on tenant edits
- approval policy primitives (`N_OF_M`, quorum, unanimous)

### 4.2 Tenant admins own

- tenant AP schema set (add/edit/retire within tenant)
- tenant overlays
- position AP bindings
- board membership and tenant approval routing
- per-user overrides

### 4.3 Selene OS runtime owns

- deterministic `ALLOW|DENY|ESCALATE` evaluation
- approval orchestration
- override application (only after approvals + simulation)
- audit/replay emission

## 5) UX Flow Rules (JD + Selene AP creation)

When user says "Create AP_CLERK" or "Create AP_CEO":

1. Selene uses NLP + LLM to classify position and draft market-baseline rules.
2. Selene asks:
   - "Do you want this on phone/desktop for review, or should I read it out loud?"
3. User may review by screen or voice.
4. Rule-by-rule actions are allowed:
   - agree
   - disagree
   - edit
   - delete
   - disable
   - add custom rule
5. Selene collects limits/escalation policy.
6. Selene presents final summary in professional writing.
7. User confirms activation.
8. Selene runs simulation gates.
9. On pass: activate new AP version + audit.
10. On fail: refuse activation with deterministic reason.

Screen-facing text rule:

- All screen text must use professional writing quality.

## 6) Approval and Board Model

Escalation policies may require:

- single approver
- `N_OF_M` approvers
- board quorum thresholds (for example 50%, 70%, unanimous)
- mixed policies (for example CFO + board quorum)

Decision closes only when policy threshold is met.

If threshold is not met within policy window: deterministic deny/expire path.

## 7) Engine Boundary Plan

### 7.1 PH1.ACCESS.001_PH2.ACCESS.002 (primary authority)

Add authority for:

- AP schema registry lifecycle
- overlay lifecycle
- board/approval policy evaluation primitives
- compiled per-user access lineage

Keep existing authority:

- gate decision (`ALLOW|DENY|ESCALATE`)
- access instance and override truth

### 7.2 PH1.POSITION

- position requires AP binding before active lifecycle transitions
- position keeps position-owned requirement schemas separate from access permissions

### 7.3 PH1.ONB

- onboarding triggers access instance compile/refresh where policy requires
- no AP rule authoring in ONB

### 7.4 PH1.BCAST + PH1.REM

- approval and board voting notifications/reminders only
- no authority decision ownership

### 7.5 PH1.J and PH1.F

- PH1.J: append-only audit proofs
- PH1.F: ledger + current projections for AP/overlay/board/access compile rows

## 8) Data Model Direction (schema-first, replay-safe)

Add AP/overlay registry model in ledger + current projection form.

Target logical groups:

1. AP definitions (ledger/current)
2. AP rule rows (ledger/current)
3. AP overlays (ledger/current)
4. board definitions + membership + vote records
5. access compiled lineage references in per-user instance

All tables must keep tenant isolation, idempotency keys, and deterministic dedupe.

## 9) Strict 8-Step Build Order

### Step 1: Docs lock (design truth)

Lock in:

- object model
- ownership
- lifecycle
- review-channel UX rules
- professional writing requirement for screen output

### Step 2: Kernel contracts lock

Add schema objects/enums for:

- AP lifecycle/versioning
- overlay ops
- board policy/voting thresholds
- access lineage refs

### Step 3: DB wiring + ECM lock

Define writes/reads and fail-closed invariants for:

- AP registry
- overlay registry
- board/vote state
- access compile lineage

### Step 4: Blueprint + simulation lock

Add/lock blueprints and simulation rows for:

- AP create/edit/activate/retire
- overlay create/edit/activate/retire
- board vote escalation flows
- access compile/refresh flows

### Step 5: Runtime gate parity

Keep gate output unchanged while replacing input resolution with schema chain.

### Step 6: Storage + repo parity

Implement deterministic ledger/current writes and compile lineage updates.

### Step 7: Test closure

Required test groups:

- deny-by-default on missing rules
- AP version pin and replay determinism
- overlay merge determinism
- position AP binding required
- escalation `N_OF_M` and board quorum
- override lifecycle types
- tenant isolation

### Step 8: Final proof + freeze checkpoint

Run readiness audit + targeted suites + workspace tests from clean tree and pin commit.

## 10) Acceptance Checklist (must all be true)

1. APs are data-driven schemas, not hard-coded permission bundles.
2. Tenant can add/edit/retire APs inside tenant scope only.
3. Position activation requires AP binding.
4. Per-user access instance stores lineage to AP/overlay versions.
5. Gate remains `ALLOW|DENY|ESCALATE` and deterministic.
6. Board/multi-approval thresholds are simulation-gated and audited.
7. Screen review can be phone/desktop or read-out-loud, with rule-by-rule agree/disagree/edit/delete/disable.
8. Screen-facing output uses professional writing quality.

## 11) Out of Scope for this packet

1. Changing onboarding requirement schema ownership model.
2. UI theme/layout redesign.
3. Non-access policy domains unrelated to AP decisions.

## 12) Execution Record

- Step 1: COMPLETED (2026-02-15)
- Step 2: COMPLETED (2026-02-15)
- Step 3: PENDING
- Step 4: PENDING
- Step 5: PENDING
- Step 6: PENDING
- Step 7: PENDING
- Step 8: PENDING

Step 1 note:
- Added this packet as canonical master-access schema scope (`docs/29_MASTER_ACCESS_SCHEMA_STRICT_FIX_PLAN_PACKET.md`).
- Updated build-plan next strict packet pointer to this packet in `docs/02_BUILD_PLAN.md`.
- Locked Step-1 design truths in this packet:
  - APs are schema records (global + tenant), not hard-coded permission bundles.
  - Screen/voice review branch is required (phone/desktop review or read-out-loud).
  - Rule-by-rule agree/disagree/edit/delete/disable/add behavior is required.
  - Screen-facing output uses professional writing quality.

Step 2 note:
- Locked kernel contract objects for master-access schema in `docs/04_KERNEL_CONTRACTS.md` via new `KC.26 PH1.ACCESS Master Access Schema Kernel Contract`.
- Added new Rust kernel contract module `crates/selene_kernel_contracts/src/ph1access.rs` and exported it in `crates/selene_kernel_contracts/src/lib.rs`.
- Step-2 object lock now includes:
  - AP lifecycle/versioning objects (`DRAFT | ACTIVE | RETIRED`, global vs tenant scope).
  - overlay operation enums/spec (`ADD_PERMISSION | REMOVE_PERMISSION | TIGHTEN_CONSTRAINT | SET_ESCALATION_POLICY`).
  - board/approval threshold policy primitives (`SINGLE_APPROVER | N_OF_M | BOARD_QUORUM_PERCENT | UNANIMOUS_BOARD | MIXED`).
  - per-user access compiled lineage refs (global AP ref, tenant AP ref, overlay refs, position ref).
- Verified gate compatibility lock remains explicit (`ALLOW | DENY | ESCALATE` unchanged externally).
- Step-2 proof:
  - `cargo test -p selene_kernel_contracts -- --nocapture` -> pass
  - `rg` checks for KC.26 and `ph1access` anchors -> pass
