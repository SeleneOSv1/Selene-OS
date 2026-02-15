Selene Kernel Contracts — v1 (Authoritative Runtime Envelopes)

This document defines the execution-grade kernel contracts that make Selene buildable, enforceable, and operable. These contracts are code-first and must be implemented as a shared kernel crate (e.g. selene_kernel_contracts). Every engine, simulation, and tool router depends on this crate. No exceptions.

Related docs:
- One-page engine inventory: `docs/07_ENGINE_REGISTRY.md`.
- Runtime wiring overview: `docs/06_ENGINE_MAP.md`.
- Behavioral laws/spec: `docs/05_OS_CONSTITUTION.md`.

The purpose of these contracts is to enforce:

single runtime authority (Selene OS),

no engine-to-engine calls,

deterministic orchestration,

auditable execution,

replayable history,

and zero drift between design and implementation.

KC.1 Global Identifiers (No Ambiguity)

All identifiers are strong types (newtypes), not raw strings.

TenantId

CorrelationId (one per end-to-end job)

TurnId (one per user turn)

WorkOrderId

ProcessId

BlueprintVersion

EngineId

CapabilityId

SimulationId

ToolId

AuditEventId

Hard rule: identifiers are never reused and never overloaded.

KC.2 EngineEnvelope (Universal Message Wrapper)

Purpose: enforce runtime mediation and prevent direct engine-to-engine calls.

EngineEnvelope (minimum fields):

schema_version

tenant_id

correlation_id

turn_id

work_order_id (optional for chat/lookup, required for processes)

source:

source_kind (OS | ENGINE | SIMULATION | TOOL_ROUTER)

source_id

destination:

engine_id

capability_id

idempotency_key (required for writes, simulations, tools)

payload (typed, schema-validated)

now (monotonic time supplied by Selene OS; used for deterministic expiry windows)

Hard rules:

Only Selene OS may send EngineEnvelope messages to engines in production mode.

Engines may only return EngineResult; they may not dispatch envelopes.

KC.3 WorkOrder (Single Truth Object)

Purpose: represent what Selene is doing, independent of phrasing.

WorkOrder:

work_order_id

tenant_id

correlation_id

intent_type

process_id

blueprint_version

requester_user_id

requester_speaker_id

device_id

session_id

status (DRAFT | CLARIFY | CONFIRM | EXECUTING | DONE | REFUSED | FAILED)

fields (typed key/value map)

evidence_refs (references only; no verbatim quotes stored in the WorkOrder)

EvidenceRef (minimum)

conversation_turn_id (PH1.F conversation_ledger)

start_byte, end_byte (byte offsets into the stored text)

field_key (optional; which field this evidence supports)

missing_fields

turn_id_next (monotonic within correlation_id; starts at 1; increments each user/selene turn)

confirmation_state (NOT_REQUIRED | PENDING | CONFIRMED | EXPIRED)

created_at

updated_at

Hard rule: WorkOrder is the only object that may cross engine boundaries for a task.

Hard rule: WorkOrder must not store raw sensitive quotes. It stores EvidenceRefs only.

KC.4 EngineResult (Engine Response Contract)

Purpose: eliminate free-form responses and enforce deterministic outcomes.

EngineResult:

schema_version

correlation_id

turn_id

work_order_id

engine_id

capability_id

status (OK | NEEDS_CLARIFY | REFUSED | FAIL)

produced_fields (typed)

missing_fields (if NEEDS_CLARIFY)

reason_code

retry_hint (NONE | RETRYABLE | NOT_RETRYABLE)

audit_event_required (bool)

payload_min (strict, bounded)

Hard rule: engines never ask users; NEEDS_CLARIFY signals are handled by PH1.X.

KC.5 ReasonCode Registry (System-Wide)

Purpose: make explainability, audit, and replay possible.

ReasonCodeRecord:

reason_code_id

owning_engine

severity (INFO | WARN | ERROR)

user_safe_template_id (for PH1.EXPLAIN)

deprecated (bool)

Hard rules:

reason codes are never reused.

deprecated codes remain valid for replay.

emitting an unknown code is a fatal error.

KC.6 Process Blueprint Record (Machine Contract)

BlueprintRecord:

process_id

version

intent_type

required_inputs

success_output_schema

ordered_steps[]

engine_id

capability_id

required_fields

produced_fields

sensitivity_level

retry_policy

confirmation_points[]

simulation_requirements[]

status (DRAFT | ACTIVE | DEPRECATED)

Hard rule: Selene OS may execute only ACTIVE blueprints.
Hard rule: `required_inputs`, `success_output_schema`, `ordered_steps`, `confirmation_points`, and `simulation_requirements` must be explicit (no `TBD`).
Hard rule: every executable blueprint step must bind explicit `engine_id + capability_id` from ACTIVE ECM and, where side effects exist, explicit `simulation_id` bindings.

KC.7 Simulation Catalog Record (Machine Contract)

SimulationRecord:

simulation_id

version

status (DRAFT | ACTIVE | LEGACY_DO_NOT_WIRE | DEPRECATED | DISABLED)

simulation_type (DRAFT | COMMIT | REVOKE)

input_schema

output_schema

required_roles

required_approvals

preconditions

postconditions

declared_side_effects

reads_tables[]

writes_tables[]

idempotency_key_rule

audit_event_codes

Hard rule: No Simulation → No Execution.
Hard rule: `required_roles`, `preconditions`, `idempotency_key_rule`, `audit_event_codes`, and table bindings must be explicit (no `TBD`).

KC.7A Engine Capability Map Record (Machine Contract)

Purpose: lock deterministic callable scope per engine and prevent hidden procedures.

EngineCapabilityMapRecord (minimum)

engine_id

version

status (DRAFT | ACTIVE | DEPRECATED | DISABLED)

owning_domain

capabilities[]

CapabilityRecord (minimum)

capability_id (stable)

name

input_schema

output_schema

allowed_callers (SELENE_OS_ONLY | SIMULATION_ONLY | OS_AND_SIMULATION)

side_effects (NONE or declared list)

reads_tables[]

writes_tables[]

idempotency_key_rule

failure_modes (OK | NEEDS_CLARIFY | REFUSED | FAIL)

reason_codes[]

audit_event_codes[]

Hard rules:

no wildcard capability ids

no "do anything" endpoints

engines expose only listed capabilities

engines never call engines directly (Selene OS orchestrates)

if `side_effects != NONE`, execution must be simulation-gated

Activation gate:

ACTIVE blueprints/simulations may reference only ACTIVE capability maps and ACTIVE capability records.

If capability binding is missing/invalid, execution fails closed.

Acceptance tests (minimum):

AT-KC-ECM-01: Unknown capability_id is rejected.

AT-KC-ECM-02: Inactive capability map cannot be executed.

AT-KC-ECM-03: Side-effect capability without simulation binding is rejected.

KC.8 Tool Contracts (Read-Only, Disciplined)

ToolRequest:

correlation_id

turn_id

tool_id

query

locale

budget (timeout_ms, max_results)

idempotency_key

ToolResult:

status (OK | FAIL)

reason_code

result_payload

provenance (timestamps, freshness, conflict_flags)

KC.9 AuditEvent (Canonical)

AuditEvent:

audit_event_id

tenant_id

correlation_id

turn_id

work_order_id

engine_id

event_type

reason_code

severity

payload_min

evidence_ref

created_at

Hard rules:

append-only

bounded payload

correlation_id required

KC.9A Audit Event Schema Lock (Item 8)

Status: `LOCKED`

Locked audit-schema rules:
- Canonical audit envelope fields are fixed and must not drift:
  `audit_event_id`, `tenant_id`, `correlation_id`, `turn_id`, `work_order_id`, `engine_id`,
  `event_type`, `reason_code`, `severity`, `payload_min`, `evidence_ref`, `created_at`.
- `reason_code` is mandatory for every emitted audit event (no silent events).
- `payload_min` must stay bounded and structured (no unbounded free-text logs).
- `audit_events` persistence contract is fixed in KC.22.9 and must remain append-only.

KC.10 IdempotencyKey (Deterministic Safety)

IdempotencyKey is derived as:

hash(tenant_id + work_order_id + operation_id + stable_input_digest)

Used for:

simulations (draft + commit)

tool calls

outbox dispatch

database writes

Hard rule: no idempotency key → no dispatch.

KC.11 Enforcement Guarantees

If these contracts are implemented and enforced:

engines cannot bypass Selene OS

retries cannot duplicate side effects

audits are complete

replay is deterministic

design cannot drift from implementation

This document is the bridge from architecture to reality.

KC.12 Policy Compiler (RBAC + ABAC → Deny-by-Default Snapshot)

KC.12.1 Mission

The Policy Compiler turns human-readable access rules into an executable, deny-by-default policy snapshot that Selene OS can evaluate deterministically at runtime.

Selene OS must never evaluate “live policy text.” It evaluates a compiled snapshot.

KC.12.2 Inputs (Policy Source of Truth)

A) Role Catalog (RBAC)

role_id

role_name

role_scope (tenant | org_unit | global)

permissions (capability-level allow list)

B) Attribute Rules (ABAC)

subject attributes (user_id, role_id, org_unit, clearance)

resource attributes (employee_id, payroll_run_id, sensitivity)

environment attributes (device_type, location_class, time_window, multi-speaker flag)

C) Process/Simulation Bindings

process_id → allowed roles

simulation_id → required roles and approvals

engine_id/capability_id → allowed caller constraints

KC.12.3 Compiler Output (Policy Snapshot)

PolicySnapshot (versioned):

policy_version_id

tenant_id

compiled_at

deny_by_default (true)

allow_rules[]

subject_match

action_match (engine/capability OR simulation OR tool)

resource_match

environment_match

decision (ALLOW)

approval_rules[]

action_match

required_approvals

redaction_rules[]

data_class

viewer_role

redaction_mode

Hard rule: runtime evaluation must be pure and deterministic: (snapshot + request) → decision.

KC.12.4 Runtime Evaluation Contract

PolicyDecision:

decision (ALLOW | DENY | REQUIRE_APPROVAL)

reason_code

required_approvals (if any)

decision_proof_hash (hash of snapshot_version + rule_id)

Selene OS must attach decision_proof_hash to audit events for replay.

KC.12.5 Deny-by-Default Rules

If no allow rule matches exactly: DENY.

If identity is unknown: DENY.

If multi-speaker present and action is sensitive: DENY or REQUIRE_APPROVAL per policy.

KC.12.6 Acceptance Tests

AT-KC-12-01: Deny by default

Scenario: action not covered by any rule.

Pass: DENY + reason_code.

AT-KC-12-02: Snapshot determinism

Scenario: same request + same snapshot.

Pass: same decision + same decision_proof_hash.

AT-KC-12-03: Approval routing

Scenario: action requires approval.

Pass: REQUIRE_APPROVAL returned with required_approvals list.

KC.13 Idempotency + Outbox (Safe Side Effects)

KC.13.1 Mission

The Outbox guarantees that retries, network failures, and restarts do not duplicate side effects. Selene must be able to “try again” safely.

KC.13.2 Outbox Record (Canonical)

OutboxRecord:

outbox_id

tenant_id

correlation_id

work_order_id

idempotency_key

operation_type (TOOL_CALL | NOTIFICATION | BROADCAST | WEB_FETCH | SIMULATION_COMMIT)

operation_payload (strict, bounded)

status (PENDING | SENT | CONFIRMED | FAILED | DEAD_LETTER)

attempt_count

next_attempt_at

last_error_reason_code

created_at

Hard rules:

(tenant_id, idempotency_key) must be unique.

writes are append-only or status-only updates; payload never mutates.

KC.13.3 Dedupe Rules

If an outbox entry with the same idempotency_key exists:

do not create a second entry

return the existing status/result reference

KC.13.4 Retry Rules (Deterministic)

Backoff schedule is deterministic (e.g., fixed sequence).

Max attempts is declared per operation_type.

After max attempts: move to DEAD_LETTER with reason_code.

KC.13.5 Restart Recovery

On restart:

scan PENDING and FAILED (retryable) entries

resume retries based on next_attempt_at

KC.13.6 Acceptance Tests

AT-KC-13-01: No duplicate side effects

Scenario: same idempotency_key dispatched twice.

Pass: only one outbox entry exists; no duplicate sends.

AT-KC-13-02: Deterministic retry

Scenario: operation fails twice then succeeds.

Pass: attempt_count increments; success recorded once.

AT-KC-13-03: Dead-letter discipline

Scenario: operation exceeds max attempts.

Pass: status=DEAD_LETTER with reason_code.

KC.13A Idempotency + Lease Contract Lock (Item 9)

Status: `LOCKED`

Locked idempotency/lease rules:
- Canonical idempotency key contract is fixed in KC.10 and applies to every retriable dispatch/write boundary.
- Outbox/dedupe contract is fixed in KC.13; `(tenant_id, idempotency_key)` uniqueness is mandatory for outbox side effects.
- WorkOrder lease single-executor contract is fixed in KC.23.4 and KC.23.5:
  - at most one ACTIVE lease per `(tenant_id, work_order_id)`
  - renew/release requires lease token ownership
  - expiry enables deterministic takeover from persisted ledger state.
- Any simulation/work-order step that performs side effects must include explicit `idempotency_key_rule`.
- No idempotency key (where retriable/side-effecting) -> no dispatch.

KC.14 Replay Tooling (Correlation → Full Deterministic Reconstruction)

KC.14.1 Mission

Replay tooling reconstructs exactly what Selene did and why, from immutable records.

Given a correlation_id, the system must be able to rebuild:

the WorkOrder progression

each gate decision

each engine result

each tool call

each simulation

each audit event

KC.14.2 Required Data for Replay

Replay requires:

audit_events with correlation_id

ledger entries referenced by evidence_ref

policy decision proof hashes

blueprint_version and simulation_version used

KC.14.3 Replay CLI Contract

ReplayCommand:

replay --tenant <tenant_id> --correlation <correlation_id>

ReplayOutput:

ordered timeline of events

gate decisions with reason codes

policy decisions with proof hash

final outcome (DONE/REFUSED/FAILED)

Hard rule: replay output must be deterministic and identical across runs.

KC.14.4 Acceptance Tests

AT-KC-14-01: Replay completeness

Scenario: full process executed.

Pass: replay shows all gates and steps in order.

AT-KC-14-02: Replay determinism

Scenario: same correlation_id replayed twice.

Pass: identical output.

KC.15 Execution Checklist (P0 → P1 → P2 Build Order)

This checklist turns the kernel spec into a running reality.

KC.15.1 P0 (Must Close First)

Freeze kernel types: EngineEnvelope, WorkOrder, EngineResult, AuditEvent, ToolRequest/Result.

Implement strict validators + schema_version enforcement.

Enforce orchestrator mediation: block direct engine-to-engine calls.

Implement PolicySnapshot evaluation (deny-by-default).

Implement idempotency keys and OutboxRecord with unique constraints.

Harden Postgres roles to prevent ledger mutation.

KC.15.2 P1 (Reliability)

Add versioning strategy for schema evolution (accept N and N-1, reject unknown).

Add deterministic retry/backoff policies per operation_type.

Add conflict-safe tool provenance fields.

Add tie-break rules for clarification selection.

KC.15.3 P2 (World-Class)

Add enforced latency SLO fields (p50/p95/p99 targets) to audit spans.

Add cost budget guardrails (per user/day).

Add multi-tenant isolation propagation (tenant_id everywhere).

Add incident controls (safe mode, kill switches) as governance simulations.

KC.15.4 Acceptance Bar (MVP Is Real)

MVP is real only if:

End-to-end wake→intent→tool path runs with correlation IDs.

No state change occurs without access + confirmation + simulation.

Outbox prevents duplication under retry.

Replay reconstructs decisions deterministically.

KC.16 PH1.F Onboarding Draft + Link Token Table Contract (Minimum)

Purpose: keep PH1.F database schema and kernel contracts fully aligned for invited onboarding.

KC.16.1 Table: onboarding_drafts

Required columns (minimum):

draft_id (PK)

tenant_id

invitee_type (COMPANY | CUSTOMER | EMPLOYEE | FAMILY_MEMBER | FRIEND | ASSOCIATE)

schema_version_id (required for EMPLOYEE and COMPANY)

creator_user_id

draft_payload_json (bounded, structured)

missing_required_fields_json (bounded, deterministic output of schema gate)

status (DRAFT_CREATED | DRAFT_READY | COMMITTED | REVOKED | EXPIRED)

created_at

updated_at

committed_entity_id (nullable)

idempotency_key (optional for retriable updates)

Unique keys / constraints:

PRIMARY KEY (draft_id)

UNIQUE (tenant_id, draft_id)

Transition enforcement rule: status transitions are monotonic and deterministic (no COMMITTED -> DRAFT_CREATED), enforced by PH1.F/PH1.LINK runtime state machine; SQL CHECK enforces enum membership only.

KC.16.2 Table: onboarding_link_tokens

Required columns (minimum):

token_id (PK)

draft_id (FK -> onboarding_drafts.draft_id)

tenant_id

token_signature

expires_at

status (DRAFT_CREATED | SENT | OPENED | ACTIVATED | CONSUMED | REVOKED | EXPIRED | BLOCKED)

bound_device_fingerprint_hash (nullable)

created_at

updated_at

consumed_at (nullable)

revoked_at (nullable)

Unique keys / constraints:

PRIMARY KEY (token_id)

UNIQUE (token_id, tenant_id)

FOREIGN KEY (draft_id) REFERENCES onboarding_drafts(draft_id)

CHECK: token contains no personal payload columns (no name/salary/ID fields)

KC.16.3 Table: onboarding_draft_write_dedupe

Required columns (minimum):

dedupe_id (PK)

scope_type (LINK | ONB)

scope_id (token_id or onboarding_session_id)

idempotency_key

write_hash

created_at

Unique keys / constraints:

PRIMARY KEY (dedupe_id)

UNIQUE (scope_type, scope_id, idempotency_key)

KC.16.4 Hard Rules

Mapping is authoritative: token_id -> draft_id must exist before send/open flows.

Invitee updates must always target the same draft_id; never fork a second draft silently.

Commit path must be atomic: create real entity + mark draft COMMITTED + mark token CONSUMED in one transaction.

Retries must be idempotent using (scope_type, scope_id, idempotency_key).

KC.17 PH1.F Reminder Table Contract (Minimum)

Purpose: keep PH1.F reminder scheduling/delivery persistence and kernel contracts aligned.

KC.17.1 Table: reminders

Required columns (minimum):

reminder_id (PK)

tenant_id

user_id

reminder_type (TASK | MEETING | TIMER | MEDICAL | CUSTOM | BCAST_MHP_FOLLOWUP)

priority_level (LOW | NORMAL | HIGH | CRITICAL)

timezone

timezone_rule (FIXED_TIMEZONE | LOCAL_TIME)

scheduled_time

recurrence_rule_json (nullable, bounded)

state (DRAFT | SCHEDULED | PRE_REMINDER_SENT | DUE_SENT | FOLLOWUP_PENDING | SNOOZED | COMPLETED | CANCELED | ESCALATED | FAILED)

quiet_hours_policy_ref (nullable)

max_attempts

attempt_count

next_attempt_at (nullable)

created_at

updated_at

completed_at (nullable)

canceled_at (nullable)

failed_at (nullable)

Unique keys / constraints:

PRIMARY KEY (reminder_id)

UNIQUE (tenant_id, reminder_id)

CHECK: state transitions are deterministic and monotonic (no COMPLETED -> SCHEDULED, no CANCELED -> SCHEDULED)

KC.17.2 Table: reminder_occurrences

Required columns (minimum):

occurrence_id (PK)

reminder_id (FK -> reminders.reminder_id)

tenant_id

occurrence_index

scheduled_time

state (SCHEDULED | PRE_REMINDER_SENT | DUE_SENT | FOLLOWUP_PENDING | SNOOZED | COMPLETED | CANCELED | ESCALATED | FAILED)

followup_time (nullable)

snooze_until (nullable)

created_at

updated_at

completed_at (nullable)

idempotency_key (optional for retriable updates)

Unique keys / constraints:

PRIMARY KEY (occurrence_id)

UNIQUE (reminder_id, occurrence_index)

UNIQUE (tenant_id, occurrence_id)

FOREIGN KEY (reminder_id) REFERENCES reminders(reminder_id)

KC.17.3 Table: reminder_delivery_attempts

Required columns (minimum):

attempt_id (PK)

reminder_id (FK -> reminders.reminder_id)

occurrence_id (FK -> reminder_occurrences.occurrence_id)

tenant_id

delivery_attempt_id (idempotency key for delivery)

delivery_channel (voice | push | text | email)

attempt_index

delivery_status (DELIVERED | DEFERRED_QUIET_HOURS | RETRY_SCHEDULED | FAIL | CHANNEL_UNAVAILABLE)

reason_code

attempted_at

next_retry_at (nullable)

delivery_proof_ref (nullable)

Unique keys / constraints:

PRIMARY KEY (attempt_id)

UNIQUE (occurrence_id, delivery_attempt_id)

UNIQUE (tenant_id, attempt_id)

FOREIGN KEY (reminder_id) REFERENCES reminders(reminder_id)

FOREIGN KEY (occurrence_id) REFERENCES reminder_occurrences(occurrence_id)

KC.17.4 Hard Rules

Delivery dedupe is mandatory: duplicate delivery_attempt_id for the same occurrence_id is a no-op.

Recurrence expansion must be bounded and deterministic.

Follow-up and escalation must write explicit state transitions (no implicit retries).

Reminder completion/cancel/fail terminal states must be final and auditable.

KC.18 PH1.F Broadcast Table Contract (Minimum)

Purpose: keep PH1.F broadcast persistence and kernel contracts aligned.

KC.18.1 Table: broadcast_envelopes

Required columns (minimum):

broadcast_id (PK)

tenant_id

sender_id

origin_context

classification (Simple | Priority | Private | Confidential | Emergency)

audience_spec_json (bounded)

delivery_policy_json (bounded)

content_payload_json (bounded, structured)

content_language

required_ack (None | Read | Confirm | Action-Confirm)

expiry_at

status (DRAFT_CREATED | SENT | HALTED | EXPIRED | CANCELED)

envelope_hash

created_at

sent_at (nullable)

updated_at

idempotency_key (optional for retriable updates)

Unique keys / constraints:

PRIMARY KEY (broadcast_id)

UNIQUE (tenant_id, broadcast_id)

CHECK: envelope fields are immutable after SENT (only allowed post-SENT mutations: status + timestamps)

KC.18.2 Table: broadcast_recipients

Required columns (minimum):

recipient_row_id (PK)

broadcast_id (FK -> broadcast_envelopes.broadcast_id)

tenant_id

recipient_id

status (Pending | Requested-Availability | Deferred | Delivered | Acknowledged | Rejected | Escalated | Expired)

privacy_choice (OutLoud | DeviceOnly | Mixed | Unknown)

rendered_language

delivery_channel_used (nullable)

attempt_count

last_attempt_at (nullable)

next_attempt_at (nullable)

ack_status (none | read | confirmed | action_confirmed)

reason_code (nullable)

created_at

updated_at

idempotency_key (optional for retriable updates)

Unique keys / constraints:

PRIMARY KEY (recipient_row_id)

UNIQUE (broadcast_id, recipient_id)

UNIQUE (tenant_id, recipient_row_id)

FOREIGN KEY (broadcast_id) REFERENCES broadcast_envelopes(broadcast_id)

KC.18.3 Table: broadcast_delivery_attempts

Required columns (minimum):

delivery_attempt_row_id (PK)

broadcast_id (FK -> broadcast_envelopes.broadcast_id)

recipient_id

tenant_id

delivery_attempt_id (idempotency key for recipient delivery)

delivery_channel (voice | push | text | email)

attempt_index

delivery_status (DELIVERED | DEFERRED | FAIL | ESCALATED)

ack_status (NONE | READ | CONFIRMED | ACTION_CONFIRMED)

reason_code

attempted_at

delivery_proof_ref (nullable)

Unique keys / constraints:

PRIMARY KEY (delivery_attempt_row_id)

UNIQUE (broadcast_id, recipient_id, delivery_attempt_id)

UNIQUE (tenant_id, delivery_attempt_row_id)

FOREIGN KEY (broadcast_id) REFERENCES broadcast_envelopes(broadcast_id)

KC.18.4 Hard Rules

If required_ack is not None, Delivered is not terminal success; success requires Acknowledged or an explicit terminal state.

Private/Confidential delivery mode must be enforceable from persisted state and auditable.

Per-recipient retry/escalation transitions must be explicit state writes.

Sender escalation events must be persisted with reason-coded linkage to recipient state.

KC.19 PH1.F Access Instance Table Contract (Minimum)

Purpose: keep `PH2.ACCESS.002` persistence ownership explicit and aligned with PH1.F.

KC.19.1 Table: access_instances

Required columns (minimum):

access_instance_id (PK)

tenant_id

user_id

role_template_id

effective_access_mode (R | W | A | X)

baseline_permissions_json (bounded)

identity_verified (bool)

verification_level (NONE | PASSCODE_TIME | BIOMETRIC | STEP_UP)

device_trust_level (DTL1 | DTL2 | DTL3 | DTL4)

lifecycle_state (RESTRICTED | ACTIVE | SUSPENDED)

policy_snapshot_ref

created_at

updated_at

idempotency_key (optional for retriable writes)

Unique keys / constraints:

PRIMARY KEY (access_instance_id)

UNIQUE (tenant_id, user_id)

UNIQUE (tenant_id, access_instance_id)

KC.19.2 Table: access_overrides

Required columns (minimum):

override_id (PK)

access_instance_id (FK -> access_instances.access_instance_id)

tenant_id

override_type (ONE_SHOT | TEMPORARY | PERMANENT | REVOKE)

scope_json (bounded)

status (ACTIVE | EXPIRED | REVOKED)

approved_by_user_id

approved_via_simulation_id

reason_code

starts_at

expires_at (nullable)

created_at

updated_at

idempotency_key (optional for retriable writes)

Unique keys / constraints:

PRIMARY KEY (override_id)

UNIQUE (tenant_id, override_id)

FOREIGN KEY (access_instance_id) REFERENCES access_instances(access_instance_id)

CHECK: no overlapping ACTIVE overrides with identical scope for the same access_instance_id

KC.19.3 Hard Rules

`PH2.ACCESS.002` owns writes to `access_instances` and `access_overrides`.

`PH1.ACCESS.001` must not write these tables directly.

All override apply/revoke operations must be simulation-gated and reason-coded.

KC.20 PH1.F Tenant + Position Table Contract (Minimum)

Purpose: align `PH1.ONB.BIZ.001` and `PH1.POSITION.001` persistence ownership with PH1.F.

KC.20.1 Table: tenant_companies

Required columns (minimum):

company_id (PK)

tenant_id

legal_name

jurisdiction

lifecycle_state (DRAFT | ACTIVE | SUSPENDED | RETIRED)

policy_shell_ref

created_at

updated_at

Unique keys / constraints:

PRIMARY KEY (company_id)

UNIQUE (tenant_id, company_id)

KC.20.2 Table: positions

Required columns (minimum):

position_id (PK)

company_id (FK -> tenant_companies.company_id)

tenant_id

position_title

department

jurisdiction

schedule_type (full_time | part_time | contract | shift)

permission_profile_ref

compensation_band_ref

lifecycle_state (Draft | Active | Suspended | Retired)

created_at

updated_at

idempotency_key (optional for retriable writes)

Unique keys / constraints:

PRIMARY KEY (position_id)

UNIQUE (tenant_id, company_id, position_title, department, jurisdiction)

UNIQUE (tenant_id, position_id)

FOREIGN KEY (company_id) REFERENCES tenant_companies(company_id)

KC.20.3 Table: position_lifecycle_events

Required columns (minimum):

event_id (PK)

tenant_id

position_id (FK -> positions.position_id)

action (CREATE_DRAFT | ACTIVATE | SUSPEND | RETIRE | POLICY_OVERRIDE)

from_state

to_state

reason_code

simulation_id

actor_user_id

created_at

Unique keys / constraints:

PRIMARY KEY (event_id)

UNIQUE (tenant_id, event_id)

FOREIGN KEY (position_id) REFERENCES positions(position_id)

KC.20.4 Hard Rules

`PH1.ONB.BIZ.001` owns writes to `tenant_companies`.

`PH1.POSITION.001` owns writes to `positions` and `position_lifecycle_events`.

Position rows store `compensation_band_ref` only; raw salary values are policy-gated and must not be embedded in link tokens.

KC.21 PH1.F Wake Table Contract (Minimum)

Purpose: align `PH1.W` wake enrollment/runtime persistence with PH1.F and simulation contracts.

KC.21.1 Table: wake_enrollment_sessions

Required columns (minimum):

wake_enrollment_session_id (PK)

user_id (FK -> identities.user_id)

device_id (FK -> devices.device_id)

onboarding_session_id (nullable FK)

wake_enroll_status (IN_PROGRESS | PENDING | COMPLETE | DECLINED)

pass_target

pass_count

attempt_count

max_attempts

enrollment_timeout_ms

reason_code (nullable)

wake_profile_id (nullable)

deferred_until (nullable)

created_at

updated_at

completed_at (nullable)

Unique keys / constraints:

PRIMARY KEY (wake_enrollment_session_id)

UNIQUE (user_id, device_id, wake_enroll_status=IN_PROGRESS)

CHECK pass_target in allowed bounds

CHECK max_attempts in allowed bounds

CHECK attempt_count <= max_attempts

KC.21.2 Table: wake_enrollment_samples

Required columns (minimum):

sample_id (PK)

wake_enrollment_session_id (FK -> wake_enrollment_sessions.wake_enrollment_session_id)

sample_seq

captured_at

sample_duration_ms

vad_coverage

snr_db

clipping_pct

rms_dbfs

noise_floor_dbfs

peak_dbfs

overlap_ratio

result (PASS | FAIL)

reason_code (nullable)

idempotency_key

Unique keys / constraints:

PRIMARY KEY (sample_id)

UNIQUE (wake_enrollment_session_id, idempotency_key)

CHECK sample_seq monotonic per wake_enrollment_session_id

KC.21.3 Table: wake_runtime_events

Required columns (minimum):

wake_event_id (PK)

session_id (nullable FK -> sessions.session_id)

user_id (nullable FK -> identities.user_id)

device_id (FK -> devices.device_id)

created_at

accepted (bool)

reason_code

wake_profile_id (nullable)

tts_active_at_trigger (bool)

media_playback_active_at_trigger (bool)

suppression_reason_code (nullable)

idempotency_key

Unique keys / constraints:

PRIMARY KEY (wake_event_id)

UNIQUE (device_id, idempotency_key)

KC.21.4 Table: wake_profile_bindings

Required columns (minimum):

user_id (FK -> identities.user_id)

device_id (FK -> devices.device_id)

wake_profile_id

artifact_version

active (bool)

created_at

updated_at

Unique keys / constraints:

UNIQUE (user_id, device_id) when active=true

KC.21.5 Hard Rules

`PH1.W` owns writes to `wake_enrollment_sessions`, `wake_enrollment_samples`, `wake_runtime_events`, and `wake_profile_bindings`.

Raw wake audio is not stored by default; only derived metrics and artifact/profile references are persisted.

`WAKE_ENROLL_*` simulations are idempotent and reason-coded.

`WAKE_ENROLL_COMPLETE_COMMIT` may persist `wake_profile_id` only when pass gates are satisfied.

KC.22 PH1.F Core Runtime Table Contract (Minimum)

Purpose: lock PH1.F core runtime tables so DB schemas per engine are explicit and aligned with ownership matrix/contracts.

KC.22.1 Table: identities

Required columns (minimum):

user_id (PK)

speaker_id (nullable)

primary_language

status (active | disabled)

created_at

Unique keys / constraints:

PRIMARY KEY (user_id)

KC.22.2 Table: devices

Required columns (minimum):

device_id (PK)

user_id (FK -> identities.user_id)

device_type

last_seen_at

audio_profile_ref (nullable)

Unique keys / constraints:

PRIMARY KEY (device_id)

FOREIGN KEY (user_id) REFERENCES identities(user_id)

KC.22.3 Table: sessions

Required columns (minimum):

session_id (PK)

user_id (FK -> identities.user_id)

device_id (FK -> devices.device_id)

session_state (OPEN | ACTIVE | SOFT_CLOSED | CLOSED | SUSPENDED)

opened_at

last_activity_at

closed_at (nullable)

Unique keys / constraints:

PRIMARY KEY (session_id)

FOREIGN KEY (user_id) REFERENCES identities(user_id)

FOREIGN KEY (device_id) REFERENCES devices(device_id)

KC.22.4 Table: preferences_current

Required columns (minimum):

user_id

preference_key

preference_value

updated_at

Unique keys / constraints:

PRIMARY KEY (user_id, preference_key)

KC.22.5 Table: preferences_ledger

Required columns (minimum):

ledger_id (PK)

user_id

event_type

key

value

evidence_ref

consent_state

created_at

idempotency_key (nullable)

Unique keys / constraints:

PRIMARY KEY (ledger_id)

KC.22.6 Table: memory_current

Required columns (minimum):

user_id

memory_key

memory_value

confidence

sensitivity_flag

last_seen_at

active (bool)

Unique keys / constraints:

PRIMARY KEY (user_id, memory_key)

KC.22.7 Table: memory_ledger

Required columns (minimum):

ledger_id (PK)

user_id

event_type

memory_key

memory_value

provenance

consent_state

created_at

idempotency_key (nullable)

Unique keys / constraints:

PRIMARY KEY (ledger_id)

KC.22.8 Table: conversation_ledger

Required columns (minimum):

conversation_turn_id (PK)

correlation_id

turn_id

session_id

user_id

device_id

role (USER | SELENE)

source (voice_transcript | typed_text | selene_output | tombstone)

text

text_hash

privacy_scope

created_at

idempotency_key (nullable)

tombstone_of_conversation_turn_id (nullable)

tombstone_reason_code (nullable)

Unique keys / constraints:

PRIMARY KEY (conversation_turn_id)

UNIQUE (correlation_id, turn_id)

KC.22.9 Table: audit_events

Required columns (minimum):

event_id (PK)

created_at

session_id

user_id (nullable)

device_id

engine

event_type

reason_code

severity

correlation_id

turn_id

payload_min

evidence_ref (nullable)

idempotency_key (nullable)

Unique keys / constraints:

PRIMARY KEY (event_id)

KC.22.10 Table: tool_cache (optional)

Required columns (minimum):

cache_id (PK)

tool_name

query_hash

locale

result_payload

expires_at

Unique keys / constraints:

PRIMARY KEY (cache_id)

KC.22.11 Table: artifacts_ledger

Required columns (minimum):

artifact_id (PK)

scope_type (TENANT | USER | DEVICE)

scope_id

artifact_type

artifact_version

package_hash

payload_ref

created_at

created_by

provenance_ref

status (ACTIVE | ROLLED_BACK | DEPRECATED)

idempotency_key (nullable)

Unique keys / constraints:

PRIMARY KEY (artifact_id)

UNIQUE (scope_type, scope_id, artifact_type, artifact_version)

KC.22.12 Hard Rules

Core ledgers are append-only: `preferences_ledger`, `memory_ledger`, `conversation_ledger`, `audit_events`, `artifacts_ledger`.

`*_current` tables are materialized current views and must be rebuildable from ledgers.

Retriable writes must use idempotency keys with deterministic dedupe.

No silent deletes; redaction uses explicit tombstones/events.

KC.23 PH1.F WorkOrder Table Contract (Minimum)

Purpose: lock WorkOrder persistence schema (ledger + current state + lease + scheduler attempts) for deterministic orchestration.

KC.23.1 Table: work_order_ledger

Required columns (minimum):

work_order_event_id (PK)

tenant_id

work_order_id

correlation_id

turn_id

event_type (WORK_ORDER_CREATED | FIELD_SET | FIELD_CONFLICT_RESOLVED | STATUS_CHANGED | STEP_STARTED | STEP_FINISHED | STEP_FAILED | STEP_RETRY_SCHEDULED | LEASE_ACQUIRED | LEASE_RENEWED | LEASE_RELEASED | WORK_ORDER_CANCELED)

work_order_status (DRAFT | CLARIFY | CONFIRM | EXECUTING | DONE | REFUSED | FAILED)

step_id (nullable)

step_status (nullable: PENDING | RUNNING | SUCCEEDED | FAILED | WAITING_RETRY | SKIPPED)

attempt_index (nullable)

timeout_ms (nullable)

max_retries (nullable)

retry_backoff_ms (nullable)

next_retry_at (nullable)

lease_owner_id (nullable)

lease_token_hash (nullable)

lease_expires_at (nullable)

payload_min (bounded, structured JSON)

idempotency_key (required for retriable writes)

created_at

Unique keys / constraints:

PRIMARY KEY (work_order_event_id)

UNIQUE (tenant_id, work_order_id, work_order_event_id)

UNIQUE (tenant_id, work_order_id, idempotency_key)

KC.23.2 Table: work_orders_current

Required columns (minimum):

work_order_id (PK)

tenant_id

correlation_id

requester_user_id

requester_speaker_id (nullable)

device_id

session_id

process_id

blueprint_version

status (DRAFT | CLARIFY | CONFIRM | EXECUTING | DONE | REFUSED | FAILED)

fields_json (bounded, structured)

missing_fields_json (bounded)

confirmation_state (NOT_REQUIRED | PENDING | CONFIRMED | EXPIRED)

turn_id_next

active_step_id (nullable)

active_step_attempt (nullable)

timeout_ms_active (nullable)

max_retries_active (nullable)

retry_backoff_ms_active (nullable)

next_retry_at (nullable)

last_failure_reason_code (nullable)

last_event_id (FK -> work_order_ledger.work_order_event_id)

created_at

updated_at

closed_at (nullable)

Unique keys / constraints:

PRIMARY KEY (work_order_id)

UNIQUE (tenant_id, work_order_id)

UNIQUE (tenant_id, correlation_id)

FOREIGN KEY (last_event_id) REFERENCES work_order_ledger(work_order_event_id)

KC.23.3 Table: work_order_step_attempts

Required columns (minimum):

step_attempt_id (PK)

tenant_id

work_order_id (FK -> work_orders_current.work_order_id)

step_id

attempt_index

status (SCHEDULED | RUNNING | SUCCEEDED | FAILED | CANCELED)

timeout_ms

max_retries

retry_backoff_ms

next_retry_at (nullable)

started_at (nullable)

finished_at (nullable)

reason_code (nullable)

idempotency_key

created_at

updated_at

Unique keys / constraints:

PRIMARY KEY (step_attempt_id)

UNIQUE (tenant_id, work_order_id, step_id, attempt_index)

UNIQUE (tenant_id, work_order_id, step_id, idempotency_key)

FOREIGN KEY (work_order_id) REFERENCES work_orders_current(work_order_id)

KC.23.4 Table: work_order_leases

Required columns (minimum):

lease_id (PK)

tenant_id

work_order_id (FK -> work_orders_current.work_order_id)

lease_owner_id

lease_token

lease_state (ACTIVE | EXPIRED | RELEASED)

lease_expires_at

acquired_at

renewed_at (nullable)

released_at (nullable)

idempotency_key (nullable)

Unique keys / constraints:

PRIMARY KEY (lease_id)

UNIQUE (tenant_id, lease_id)

UNIQUE (tenant_id, lease_token)

UNIQUE (tenant_id, work_order_id, idempotency_key)

FOREIGN KEY (work_order_id) REFERENCES work_orders_current(work_order_id)

KC.23.5 Hard Rules

`work_order_ledger` is append-only; corrections are new events.

`work_orders_current` is a materialized current view and must be rebuildable from `work_order_ledger`.

Any write to `work_orders_current` must be paired with a corresponding `work_order_ledger` event in the same transaction.

Step retries must be deterministic and auditable via `work_order_step_attempts` + `work_order_ledger`.

At most one ACTIVE lease may exist per `(tenant_id, work_order_id)` at a time (enforced by index/constraint strategy).

KC.24 PH1.F Capability Request Table Contract (Minimum)

Purpose: lock `PH1.CAPREQ` persistence schema as append-only ledger + rebuildable current projection.

KC.24.1 Table: capreq_ledger

Required columns (minimum):

capreq_event_id (PK)

tenant_id

capreq_id

requester_user_id (FK -> identities.user_id)

action (CREATE_DRAFT | SUBMIT_FOR_APPROVAL | APPROVE | REJECT | FULFILL | CANCEL)

status (DRAFT | PENDING_APPROVAL | APPROVED | REJECTED | FULFILLED | CANCELED)

reason_code

payload_hash

created_at

idempotency_key (optional for retriable writes)

Unique keys / constraints:

PRIMARY KEY (capreq_event_id)

UNIQUE (tenant_id, capreq_id, idempotency_key)

FOREIGN KEY (requester_user_id) REFERENCES identities(user_id)

KC.24.2 Table: capreq_current

Required columns (minimum):

tenant_id

capreq_id

requester_user_id (FK -> identities.user_id)

status (DRAFT | PENDING_APPROVAL | APPROVED | REJECTED | FULFILLED | CANCELED)

last_action

payload_hash

source_event_id (FK -> capreq_ledger.capreq_event_id)

updated_at

last_reason_code

Unique keys / constraints:

PRIMARY KEY (tenant_id, capreq_id)

UNIQUE (tenant_id, capreq_id, source_event_id)

FOREIGN KEY (source_event_id) REFERENCES capreq_ledger(capreq_event_id)

FOREIGN KEY (requester_user_id) REFERENCES identities(user_id)

KC.24.3 Hard Rules

`capreq_ledger` is append-only; overwrite attempts must fail closed.

`capreq_current` is materialized state and must be rebuildable from `capreq_ledger`.

All CAPREQ retriable writes require deterministic idempotency keys and tenant-scoped dedupe.

`PH1.CAPREQ` owns writes to `capreq_ledger` and `capreq_current`; other engines read-only unless explicit simulation contract allows otherwise.

KC.25 PH1.F Position Requirements Schema + Onboarding Backfill Table Contract (Minimum)

Purpose: lock position-owned requirements schema truth and deterministic onboarding backfill campaign persistence.

KC.25.1 Table: position_requirements_schema_ledger

Required columns (minimum):

schema_event_id (PK)

tenant_id

company_id

position_id

schema_version_id

action (CREATE_DRAFT | UPDATE_COMMIT | ACTIVATE_COMMIT | RETIRE_COMMIT)

selector_snapshot_json (bounded)

field_specs_json (bounded, typed field specs)

overlay_ops_json (bounded, optional)

reason_code

actor_user_id (FK -> identities.user_id)

created_at

idempotency_key (optional for retriable writes)

Unique keys / constraints:

PRIMARY KEY (schema_event_id)

UNIQUE (tenant_id, position_id, schema_version_id, action, idempotency_key)

FOREIGN KEY (actor_user_id) REFERENCES identities(user_id)

KC.25.2 Table: position_requirements_schema_current

Required columns (minimum):

tenant_id

company_id

position_id

active_schema_version_id

active_selector_snapshot_json (bounded)

active_field_specs_json (bounded, typed field specs)

source_event_id (FK -> position_requirements_schema_ledger.schema_event_id)

updated_at

last_reason_code

Unique keys / constraints:

PRIMARY KEY (tenant_id, position_id)

UNIQUE (tenant_id, position_id, source_event_id)

FOREIGN KEY (source_event_id) REFERENCES position_requirements_schema_ledger(schema_event_id)

KC.25.3 Table: onboarding_requirement_backfill_campaigns

Required columns (minimum):

campaign_id (PK)

tenant_id

company_id

position_id

schema_version_id

rollout_scope (NewHiresOnly | CurrentAndNew)

state (DRAFT_CREATED | RUNNING | COMPLETED | CANCELED)

created_by_user_id (FK -> identities.user_id)

reason_code

created_at

updated_at

completed_at (nullable)

idempotency_key (optional for retriable writes)

Unique keys / constraints:

PRIMARY KEY (campaign_id)

UNIQUE (tenant_id, campaign_id)

UNIQUE (tenant_id, position_id, schema_version_id, idempotency_key)

FOREIGN KEY (created_by_user_id) REFERENCES identities(user_id)

KC.25.4 Table: onboarding_requirement_backfill_targets

Required columns (minimum):

target_row_id (PK)

campaign_id (FK -> onboarding_requirement_backfill_campaigns.campaign_id)

tenant_id

user_id (FK -> identities.user_id)

status (PENDING | REQUESTED | REMINDED | COMPLETED | EXEMPTED | FAILED)

missing_fields_json (bounded)

last_reason_code

created_at

updated_at

completed_at (nullable)

idempotency_key (optional for retriable updates)

Unique keys / constraints:

PRIMARY KEY (target_row_id)

UNIQUE (campaign_id, user_id)

UNIQUE (tenant_id, target_row_id)

FOREIGN KEY (campaign_id) REFERENCES onboarding_requirement_backfill_campaigns(campaign_id)

FOREIGN KEY (user_id) REFERENCES identities(user_id)

KC.25.5 Hard Rules

`position_requirements_schema_ledger` is append-only; active schema state is rebuilt from ledger events.

`PH1.POSITION` owns schema truth and writes for requirements schema lifecycle events.

`PH1.ONB` executes pinned active schema only and must not mutate schema definitions.

`PH1.LINK` may provide selector-hint input only and must not write or mutate schema definitions.

Backfill campaigns are explicit, simulation-gated workflows; no implicit retroactive schema enforcement is allowed.

For `CurrentAndNew` rollout scope, target population selection and progress state must be deterministic and auditable via campaign + target rows.
