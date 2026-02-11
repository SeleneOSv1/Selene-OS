# Selene DB Ownership Matrix (Authoritative)

Purpose:
- define which engines may read/write which PH1.F tables
- prevent hidden cross-engine DB writes
- make simulation-to-database wiring deterministic and reviewable

Hard rules:
- All writes are simulation-gated (`DRAFT`/`COMMIT`/`REVOKE`) and audit-logged.
- No engine may write outside its declared table scope.
- `PH1.F` owns schema/migrations/invariants; other engines own business logic only.
- Retries must be idempotent (dedupe keys required for retriable writes).
- No cross-tenant reads/writes.

Lock status source:
- Design-lock status is canonical in `docs/11_DESIGN_LOCK_SEQUENCE.md`.
- This document is canonical only for DB ownership rows, table inventory, and write-scope boundaries.

## Table Inventory (PH1.F)

| table | status | purpose |
|---|---|---|
| `identities` | ACTIVE | stable user identity rows |
| `devices` | ACTIVE | device records + trust/context |
| `sessions` | ACTIVE | session lifecycle state |
| `preferences_current` | ACTIVE | current preference materialized view |
| `preferences_ledger` | ACTIVE | append-only preference history |
| `memory_current` | ACTIVE | current memory materialized view |
| `memory_ledger` | ACTIVE | append-only memory history |
| `conversation_ledger` | ACTIVE | append-only user/selene turns |
| `audit_events` | ACTIVE | append-only system proof trail |
| `tool_cache` | ACTIVE (optional) | read-only helper cache |
| `artifacts_ledger` | ACTIVE | append-only model/policy artifact lifecycle |
| `wake_enrollment_sessions` | ACTIVE | wake enrollment session state machine (in_progress/pending/complete) |
| `wake_enrollment_samples` | ACTIVE | wake enrollment sample quality ledger (pass/fail reason-coded) |
| `wake_runtime_events` | ACTIVE | wake runtime accept/reject/suppress event trail |
| `wake_profile_bindings` | ACTIVE | current wake profile binding by (user_id, device_id) |
| `onboarding_drafts` | ACTIVE | creator/invitee onboarding draft state |
| `onboarding_link_tokens` | ACTIVE | minimal token records mapped to drafts |
| `onboarding_draft_write_dedupe` | ACTIVE | idempotent dedupe for onboarding writes |
| `reminders` | ACTIVE | reminder header/state machine row |
| `reminder_occurrences` | ACTIVE | expanded recurrence + occurrence state |
| `reminder_delivery_attempts` | ACTIVE | delivery proof + retry attempts |
| `broadcast_envelopes` | ACTIVE | immutable broadcast envelope draft/sent |
| `broadcast_recipients` | ACTIVE | per-recipient state machine row |
| `broadcast_delivery_attempts` | ACTIVE | delivery/ack/escalation proof |
| `access_instances` | ACTIVE | per-user access engine instance truth (PH2.ACCESS.002) |
| `access_overrides` | ACTIVE | AP-approved override lifecycle rows (PH2.ACCESS.002) |
| `tenant_companies` | ACTIVE | business onboarding company/tenant records |
| `positions` | ACTIVE | position truth (title/scope/permission profile/comp band ref) |
| `position_lifecycle_events` | ACTIVE | append-only position state transitions |
| `capreq_ledger` | ACTIVE | append-only capability request lifecycle truth |
| `capreq_current` | ACTIVE | capability request current projection (rebuildable from ledger) |
| `work_order_ledger` | ACTIVE | append-only orchestration work order truth |
| `work_orders_current` | ACTIVE | materialized current work order state (rebuildable from ledger) |
| `work_order_step_attempts` | ACTIVE | deterministic step retry/scheduler attempt history |
| `work_order_leases` | ACTIVE | work order lease ownership rows |
| `simulation_catalog` | ACTIVE | append-only simulation definition ledger (versioned records) |
| `simulation_catalog_current` | ACTIVE | current simulation definition projection (rebuildable from ledger) |
| `engine_capability_maps` | ACTIVE | append-only engine capability map ledger (versioned records) |
| `engine_capability_maps_current` | ACTIVE | current engine capability map projection (rebuildable from ledger) |
| `governance_definitions` | PROPOSED | active/draft blueprint-simulation definition snapshots (enterprise) |
| `export_jobs` | PROPOSED | compliance export job state/proof rows (enterprise) |
| `review_cases` | PROPOSED | policy-required human review routing rows (enterprise) |

## Schema Contract Bindings (Item 3)

Hard rule:
- Every `ACTIVE` table must map to an explicit PH1.F table contract section in `docs/04_KERNEL_CONTRACTS.md` and/or `docs/05_OS_CONSTITUTION.md`.
- `PROPOSED` tables remain pending under later lock items and must not be treated as active runtime schema.

| table | contract_ref |
|---|---|
| `identities` | `docs/04_KERNEL_CONTRACTS.md` KC.22.1, `docs/05_OS_CONSTITUTION.md` F.4.1 |
| `devices` | `docs/04_KERNEL_CONTRACTS.md` KC.22.2, `docs/05_OS_CONSTITUTION.md` F.4.2 |
| `sessions` | `docs/04_KERNEL_CONTRACTS.md` KC.22.3, `docs/05_OS_CONSTITUTION.md` F.4.3 |
| `preferences_current` | `docs/04_KERNEL_CONTRACTS.md` KC.22.4, `docs/05_OS_CONSTITUTION.md` F.4.4 |
| `preferences_ledger` | `docs/04_KERNEL_CONTRACTS.md` KC.22.5, `docs/05_OS_CONSTITUTION.md` F.4.5 |
| `memory_current` | `docs/04_KERNEL_CONTRACTS.md` KC.22.6, `docs/05_OS_CONSTITUTION.md` F.4.6 |
| `memory_ledger` | `docs/04_KERNEL_CONTRACTS.md` KC.22.7, `docs/05_OS_CONSTITUTION.md` F.4.7 |
| `conversation_ledger` | `docs/04_KERNEL_CONTRACTS.md` KC.22.8, `docs/05_OS_CONSTITUTION.md` F.4.8 |
| `audit_events` | `docs/04_KERNEL_CONTRACTS.md` KC.22.9, `docs/05_OS_CONSTITUTION.md` F.4.9 |
| `tool_cache` | `docs/04_KERNEL_CONTRACTS.md` KC.22.10, `docs/05_OS_CONSTITUTION.md` F.4.10 |
| `artifacts_ledger` | `docs/04_KERNEL_CONTRACTS.md` KC.22.11, `docs/05_OS_CONSTITUTION.md` F.4.11 |
| `onboarding_drafts` | `docs/04_KERNEL_CONTRACTS.md` KC.16.1, `docs/05_OS_CONSTITUTION.md` F.4.12 |
| `onboarding_link_tokens` | `docs/04_KERNEL_CONTRACTS.md` KC.16.2, `docs/05_OS_CONSTITUTION.md` F.4.12 |
| `onboarding_draft_write_dedupe` | `docs/04_KERNEL_CONTRACTS.md` KC.16.3, `docs/05_OS_CONSTITUTION.md` F.4.12 |
| `reminders` | `docs/04_KERNEL_CONTRACTS.md` KC.17.1, `docs/05_OS_CONSTITUTION.md` F.4.13 |
| `reminder_occurrences` | `docs/04_KERNEL_CONTRACTS.md` KC.17.2, `docs/05_OS_CONSTITUTION.md` F.4.13 |
| `reminder_delivery_attempts` | `docs/04_KERNEL_CONTRACTS.md` KC.17.3, `docs/05_OS_CONSTITUTION.md` F.4.13 |
| `broadcast_envelopes` | `docs/04_KERNEL_CONTRACTS.md` KC.18.1, `docs/05_OS_CONSTITUTION.md` F.4.14 |
| `broadcast_recipients` | `docs/04_KERNEL_CONTRACTS.md` KC.18.2, `docs/05_OS_CONSTITUTION.md` F.4.14 |
| `broadcast_delivery_attempts` | `docs/04_KERNEL_CONTRACTS.md` KC.18.3, `docs/05_OS_CONSTITUTION.md` F.4.14 |
| `access_instances` | `docs/04_KERNEL_CONTRACTS.md` KC.19.1, `docs/05_OS_CONSTITUTION.md` F.4.15 |
| `access_overrides` | `docs/04_KERNEL_CONTRACTS.md` KC.19.2, `docs/05_OS_CONSTITUTION.md` F.4.15 |
| `tenant_companies` | `docs/04_KERNEL_CONTRACTS.md` KC.20.1, `docs/05_OS_CONSTITUTION.md` F.4.16 |
| `positions` | `docs/04_KERNEL_CONTRACTS.md` KC.20.2, `docs/05_OS_CONSTITUTION.md` F.4.16 |
| `position_lifecycle_events` | `docs/04_KERNEL_CONTRACTS.md` KC.20.3, `docs/05_OS_CONSTITUTION.md` F.4.16 |
| `capreq_ledger` | `docs/04_KERNEL_CONTRACTS.md` KC.24.1, `docs/05_OS_CONSTITUTION.md` F.4.19 |
| `capreq_current` | `docs/04_KERNEL_CONTRACTS.md` KC.24.2, `docs/05_OS_CONSTITUTION.md` F.4.19 |
| `wake_enrollment_sessions` | `docs/04_KERNEL_CONTRACTS.md` KC.21.1, `docs/05_OS_CONSTITUTION.md` F.4.17 |
| `wake_enrollment_samples` | `docs/04_KERNEL_CONTRACTS.md` KC.21.2, `docs/05_OS_CONSTITUTION.md` F.4.17 |
| `wake_runtime_events` | `docs/04_KERNEL_CONTRACTS.md` KC.21.3, `docs/05_OS_CONSTITUTION.md` F.4.17 |
| `wake_profile_bindings` | `docs/04_KERNEL_CONTRACTS.md` KC.21.4, `docs/05_OS_CONSTITUTION.md` F.4.17 |
| `work_order_ledger` | `docs/04_KERNEL_CONTRACTS.md` KC.23.1, `docs/05_OS_CONSTITUTION.md` F.4.18 |
| `work_orders_current` | `docs/04_KERNEL_CONTRACTS.md` KC.23.2, `docs/05_OS_CONSTITUTION.md` F.4.18 |
| `work_order_step_attempts` | `docs/04_KERNEL_CONTRACTS.md` KC.23.3, `docs/05_OS_CONSTITUTION.md` F.4.18 |
| `work_order_leases` | `docs/04_KERNEL_CONTRACTS.md` KC.23.4, `docs/05_OS_CONSTITUTION.md` F.4.18 |
| `simulation_catalog` | `docs/04_KERNEL_CONTRACTS.md` KC.7, `docs/05_OS_CONSTITUTION.md` SCS.3 |
| `simulation_catalog_current` | `docs/04_KERNEL_CONTRACTS.md` KC.7, `docs/05_OS_CONSTITUTION.md` SCS.3 |
| `engine_capability_maps` | `docs/04_KERNEL_CONTRACTS.md` KC.7A, `docs/05_OS_CONSTITUTION.md` ECM.1..ECM.8 |
| `engine_capability_maps_current` | `docs/04_KERNEL_CONTRACTS.md` KC.7A, `docs/05_OS_CONSTITUTION.md` ECM.1..ECM.8 |
| `governance_definitions` | PENDING (enterprise definition-set snapshot schema lock) |
| `export_jobs` | PENDING (enterprise schema lock phase) |
| `review_cases` | PENDING (enterprise schema lock phase) |

## Engine -> DB Rights Matrix

| engine | read tables | write tables | must never write |
|---|---|---|---|
| `PH1.F` | all | schema-managed writes from approved contracts | N/A |
| `PH1.J` (Audit) | `audit_events` (read for replay checks) | `audit_events` | business state tables |
| `PH1.M` (Memory) | `memory_current`, `memory_ledger`, `conversation_ledger` | `memory_ledger`, `memory_current` (materialized), redaction markers via policy | access/role tables, onboarding token tables |
| `PH1.PERSONA` (Personalization Profile) | `memory_current`, `memory_ledger`, `conversation_ledger`, `audit_events` | none directly (audit via PH1.J) | direct writes to memory/access/onboarding tables |
| `PH1.FEEDBACK` (Correction & Confidence Feedback) | `identities`, `devices`, `sessions`, `audit_events` | `audit_events` (feedback signal events) | memory/access/onboarding/reminder/broadcast tables |
| `PH1.LEARN` (Learning Ledger & Packaging) | `audit_events`, `artifacts_ledger`, `identities` (user-scope checks) | `artifacts_ledger` (versioned adaptation artifacts) | access/authority/session/work-order runtime tables |
| `PH1.KNOW` (Tenant Dictionary Packs) | `artifacts_ledger` | `artifacts_ledger` (tenant vocabulary/pronunciation packs) | cross-tenant artifacts, access/runtime state tables |
| `PH1.L` (Session Lifecycle) | `sessions`, `devices` | `sessions` | onboarding, reminder, broadcast tables |
| `PH1.W` (Wake) | `devices`, `sessions`, `wake_profile_bindings` | `wake_enrollment_sessions`, `wake_enrollment_samples`, `wake_runtime_events`, `wake_profile_bindings` | onboarding/access/reminder/broadcast tables |
| `PH1.LINK` | `onboarding_drafts`, `onboarding_link_tokens`, tenant schema refs | `onboarding_drafts`, `onboarding_link_tokens`, `onboarding_draft_write_dedupe` | memory/audit direct mutation (audit via PH1.J only) |
| `PH1.ONB.ORCH.001` | `onboarding_drafts`, `onboarding_link_tokens`, `tenant_companies`, `positions`, schema refs | `onboarding_drafts`, `onboarding_link_tokens`, `onboarding_draft_write_dedupe` | direct access override tables |
| `PH1.REM.001` | `reminders`, `reminder_occurrences`, quiet-hour settings refs | `reminders`, `reminder_occurrences`, `reminder_delivery_attempts` | onboarding/access tables |
| `PH1.BCAST.001` | `broadcast_envelopes`, `broadcast_recipients`, recipient prefs | `broadcast_envelopes`, `broadcast_recipients`, `broadcast_delivery_attempts` | onboarding/access tables |
| `PH1.ONB.BIZ.001` | `tenant_companies`, tenant policy refs | `tenant_companies` | access/memory/reminder/broadcast tables |
| `PH1.POSITION.001` | `tenant_companies`, `positions`, policy refs | `positions`, `position_lifecycle_events` | access instance tables, reminder/broadcast tables |
| `PH1.CAPREQ` | `access_instances`, `access_overrides`, `capreq_current` | `capreq_ledger`, `capreq_current` | direct permission-truth mutation outside Access/Authority flows |
| `PH1.ACCESS.001` | policy snapshots, `PH2.ACCESS.002` refs | none directly (gate decisions + escalation requests) | all PH1.F business tables |
| `PH2.ACCESS.002` | `access_instances`, `access_overrides`, role/policy refs | `access_instances`, `access_overrides` | onboarding/memory/reminder/broadcast tables |
| `PH1.X` | read-only context from memory/session/audit summaries | none directly | any PH1.F table |
| `PH1.WORK` (enterprise) | `work_order_ledger`, `work_orders_current`, `work_order_step_attempts`, `work_order_leases` | `work_order_ledger`, `work_orders_current`, `work_order_step_attempts` | tenant/access core tables |
| `PH1.LEASE` (enterprise) | `work_order_leases`, `work_order_ledger`, `work_orders_current` | `work_order_leases` | onboarding/access/memory tables |
| `PH1.GOV` (enterprise) | `simulation_catalog`, `simulation_catalog_current`, `engine_capability_maps`, `engine_capability_maps_current`, `governance_definitions` | `simulation_catalog`, `simulation_catalog_current`, `engine_capability_maps`, `engine_capability_maps_current`, `governance_definitions` | business runtime tables directly |
| `PH1.EXPORT` (enterprise) | `audit_events`, `conversation_ledger`, `work_order_ledger`, `work_orders_current` | `export_jobs` | core business tables |
| `PH1.REVIEW` (enterprise) | `review_cases`, policy refs | `review_cases` | access/position/memory tables |
| `Simulation Executor` | table scope declared by simulation contract only | writes only via declared simulation contract | undeclared tables |

## Simulation-to-DB Binding Rules

Every simulation record must declare directly, or via an approved domain binding profile in `docs/08_SIMULATION_CATALOG.md`:
- `reads_tables[]`
- `writes_tables[]`
- `idempotency_key_rule`
- `preconditions` (including access + confirmation requirements)
- `postconditions` (including state transitions)
- required `audit_events`

Activation gate:
- no simulation becomes `ACTIVE` unless table bindings are explicit and kernel-valid.

## New Simulation Checklist (DB Safety)

- [ ] Blueprint exists and is `ACTIVE`.
- [ ] Simulation has deterministic input/output schemas.
- [ ] `reads_tables[]` and `writes_tables[]` are declared (directly or via profile).
- [ ] Write scope matches this matrix (no out-of-scope writes).
- [ ] Idempotency key rule is explicit.
- [ ] Atomic transaction boundary is explicit for COMMIT flows.
- [ ] Required audit events are explicit.
- [ ] Tenant boundary checks are explicit.
- [ ] Replay path is deterministic.
- [ ] Failure path does not leave partial state.

## Current Closure Status

1. `TBD` contract placeholders are disallowed for active simulation contracts.
2. Simulation DB bindings must exist directly in records or in approved domain profiles in `docs/08_SIMULATION_CATALOG.md`.
3. Activation gate: any simulation with missing contract bindings remains `DRAFT` and cannot move to `ACTIVE`.
