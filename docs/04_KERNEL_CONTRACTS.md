Selene Kernel Contracts — v1 (Authoritative Runtime Envelopes)

This document defines the execution-grade kernel contracts that make Selene buildable, enforceable, and operable. These contracts are code-first and must be implemented as a shared kernel crate (e.g. selene_kernel_contracts). Every engine, simulation, and tool router depends on this crate. No exceptions.

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

created_at

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

evidence_spans (verbatim transcript excerpts + transcript_hash)

missing_fields

confirmation_state (NOT_REQUIRED | PENDING | CONFIRMED | EXPIRED)

created_at

updated_at

Hard rule: WorkOrder is the only object that may cross engine boundaries for a task.

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

audit_required (bool)

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

KC.7 Simulation Catalog Record (Machine Contract)

SimulationRecord:

simulation_id

version

status (DRAFT | ACTIVE | DEPRECATED | DISABLED)

simulation_type (DRAFT | COMMIT | REVOKE)

input_schema

output_schema

required_roles

required_approvals

preconditions

postconditions

declared_side_effects

idempotency_key_rule

audit_event_codes

Hard rule: No Simulation → No Execution.

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
--- thi sis the firs tdocument - i will paste the second next 

