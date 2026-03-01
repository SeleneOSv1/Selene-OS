# Agent Execution Core (AEC) Build Plan

Status: Design Only (No Code)

Depends On:
- `37_AGENT_SIM_FINDER_CORE_BUILD_PLAN.md`
- `35_AUTO_MICRO_BUILD_PLAN.md` (optimization layer later)
- `PH1.NLP / PH1.X / SimulationExecutor / PH2.ACCESS`

## 0. Purpose

Agent Execution Core (AEC) is not a new execution authority.

It is:
- A deterministic, simulation-orchestrating planner that converts multi-step goals into governed simulation dispatches.

It must:
- Decompose goals into simulation steps.
- Relay clarify missing data (one-question rule from Finder).
- Enforce confirm + access + ACTIVE simulation gating.
- Execute step-by-step via `SimulationExecutor`.
- Persist append-only audit trail.
- Never bypass `No Simulation -> No Execution`.

## 1. Non-Goals (Hard Stop)

The Agent Core must NOT:
- Execute money transfers without simulation + access.
- Grant permissions.
- Modify schemas.
- Promote artifacts.
- Bypass builder governance.
- Trigger `AUTO_MICRO` actions.
- Run unbounded loops.
- Execute side effects without ACTIVE simulation.
- Mutate PH1.F business-state tables directly (audit/proof appends via governed storage APIs are allowed).

Agent is orchestration only.

## 2. Architectural Placement

`PH1.NLP -> PH1.SIM_FINDER -> PH1.AGENT_EXECUTION_CORE -> SimulationExecutor (deterministic)`

Agent must:
- Use Simulation Finder to identify simulation candidates.
- Never construct raw simulation IDs manually.
- Always dispatch through `SimulationExecutor`.

### 2.1 Runtime ownership boundary (single source of truth)

- PH1.X runtime is the turn-level host that invokes Finder and Execution Core.
- Finder owns simulation candidate ranking + terminal packet emission.
- Execution Core owns plan construction, confirm/access/ACTIVE precheck orchestration, and dispatch envelope construction.
- SimulationExecutor owns side effects, execution commits, and final authoritative access/ACTIVE hard-gate enforcement.

No component may duplicate ranking authority or bypass this boundary.

### 2.2 Interfaces (Explicit Contracts)

#### 37_SIM_FINDER -> 38_AGENT_EXECUTION_CORE

Input to Execution Core from Simulation Finder is exactly one packet per cycle:
- `SimulationMatchPacket`
- `ClarifyPacket`
- `RefusePacket`
- `MissingSimulationPacket`

Execution Core required behavior by packet type:
- `SimulationMatchPacket` -> consume Finder top-1 candidate as-is; in Phase 1 build deterministic single-step `AgentExecutionPlan`, and in Phase 2 (`Run A2+`) build deterministic multi-step plan via template registry; continue through confirm/access/ACTIVE checks.
- `ClarifyPacket` -> surface exactly one clarify question and stop dispatch path.
- `RefusePacket` -> return deterministic refusal response and stop dispatch path.
- `MissingSimulationPacket` -> route to Dev Intake fallback path (no dispatch attempt).

Hard rule:
- Execution Core must not re-rank, re-score, or replace Finder candidate simulations.

#### 38_AGENT_EXECUTION_CORE -> SimulationExecutor

Execution Core output to `SimulationExecutor` must be a bounded dispatch envelope:
- `correlation_id`
- `tenant_id`
- `user_id`
- `step_id`
- `simulation_id` (must come from Finder packet, not synthesized)
- `required_fields`
- `confirmation_proof_ref` (when required)
- `access_action`
- `idempotency_key`
- `policy_snapshot_ref`
- `artifact_fingerprint_bundle_ref`
- `rollback_plan_ref` (required for impactful simulation classes)

SimulationExecutor returns:
- deterministic simulation outcome payload
- reason code
- audit/proof refs

Execution Core must persist and return those outputs without mutation.

## 3. Execution Model

### 3.1 Single-Step Baseline (Phase 1)

Initial capability:
- One simulation per agent cycle.
- Confirm required before dispatch (if impactful).
- Return result.
- End.

No background chaining yet.

### 3.2 Future Extension (Phase 2)

Multi-step deterministic chain:
- Agent builds explicit plan graph with deterministic `step_id`s.
- Executes one simulation per step in strict sequence.
- Confirms each high-risk/impactful step before dispatch.
- Persists step state as DB truth for resumable replay.
- Uses deterministic retry policy and deterministic rollback pointers.
- Aborts chain on first non-recoverable failure.

Phase 1 remains single-step only.

## 4. Core Data Contracts

### 4.1 AgentGoalRequest

Fields:
- `correlation_id`
- `tenant_id`
- `user_id`
- `thread_id`
- `raw_user_utterance`
- `normalized_intent`
- `sim_finder_packet`
- `confirm_state`
- `idempotency_key`
- `attempt_counter`

### 4.2 AgentExecutionPlan

Fields:
- `plan_id`
- `finder_packet_ref`
- `ordered_steps[]`
- `required_confirmations[]`
- `risk_level`
- `access_actions_required[]`
- `policy_snapshot_ref`
- `created_at`

### 4.3 AgentStep

Fields:
- `step_id`
- `simulation_id`
- `required_fields[]`
- `required_fields_fingerprint`
- `policy_snapshot_version`
- `confirmation_required`
- `access_action`
- `idempotency_key_recipe`
- `retry_policy`
- `rollback_plan_ref`

### 4.4 AgentExecutionOutcome

Fields:
- `step_id`
- `simulation_outcome`
- `audit_event_id`
- `proof_refs[]`
- `dispatch_decision_hash`
- `next_step` (if any)
- `reason_code`
- `user_visible_text`

### 4.5 Output ownership boundary

- `user_visible_text` is an execution outcome payload only.
- Transcript persistence/display ownership remains outside AEC (adapter transcript path + PH1.F conversation ledger).
- AEC must not write transcript rows directly.

## 5. Deterministic Planning Rules

Agent must:
- Only accept simulations returned by Simulation Finder.
- Consume Finder terminal packets without re-ranking.
- Build execution plan only from `SimulationMatchPacket`.
- Stop planning on `ClarifyPacket`, `RefusePacket`, or `MissingSimulationPacket`.

No random selection.

### 5.1 Dev Intake Fallback (Hard Rule)

If Simulation Finder returns `MissingSimulationPacket`, Execution Core must:
- Not attempt any simulation dispatch.
- Not call `SimulationExecutor`.
- Route to Dev Intake pipeline deterministically.
- Return user-visible escalation status only.

No exception path is allowed for this rule.

## 6. Clarify Discipline

One-question rule is Finder-owned and relay-only in Execution Core.

Agent must:
- Relay `ClarifyPacket.question` unchanged.
- Increase `attempt_counter` using Finder packet metadata.
- Stop dispatch path for that cycle.
- Never synthesize a second clarify question.

## 7. Confirmation Rules

If simulation is:
- impactful
- side-effecting
- irreversible

Agent must:
- Emit confirmation prompt.
- Await explicit YES.
- Reject ambiguous confirmation.

Confirmation parsing must be deterministic and multilingual-safe:
- use a versioned per-language confirmation lexicon (`confirm_lexicon_version_ref`)
- accept only canonical affirmative tokens from that lexicon
- reject ambiguous acknowledgements (for example: "okay", "fine", "maybe later")
- persist `confirmation_parse_proof_ref` with accepted token and language tag

No confirm, no dispatch.

## 8. Access + ACTIVE Enforcement

Before dispatch (precheck layer):
- Access engine lookup (per-user instance only).
- Ensure action is allowed or requires AP.
- Ensure simulation is ACTIVE for tenant.

Fail closed if:
- `SIMULATION_NOT_REGISTERED`
- `SIMULATION_NOT_ACTIVE`
- `ACCESS_SCOPE_VIOLATION`
- `ACCESS_AP_REQUIRED`

Final gate law:
- SimulationExecutor must re-enforce access + ACTIVE checks as final authoritative hard gate before side effects.

## 9. Idempotency

Canonical source of truth:
- Section `19.1` (`Canonical idempotency key recipes`) is authoritative for all AEC write paths.

Phase 1 compatibility key:
- single-step dispatch may use the compatibility projection key
  - `correlation_id + step_id + simulation_id + required_fields_fingerprint + policy_snapshot_version`
- this compatibility key must map to the canonical `agent_step_execute:*` recipe shape in `19.1`.

Rules:
- Use append-only ledgers.
- Be retry-safe.
- Prevent duplicate side effects.
- Do not introduce additional idempotency recipes outside Section `19.1`.

Acceptance tests required.

## 10. Audit Requirements

Every step must append:
- `PH1.AGENT_EXECUTION` ledger row.

Each row links to:
- simulation ledger entry
- access decision entry
- confirmation entry
- rollback ref

Audit chain must be reconstructable.

### 10.1 PH1.F storage contracts (DB-truth)

Canonical multi-step tables (authoritative):
- `agent_plan_ledger` (append-only plan lifecycle events)
- `agent_plan_current` (projection per plan)
- `agent_step_ledger` (append-only step lifecycle/execution events)
- `agent_step_current` (projection per plan/step)
- `agent_idempotency_index` (idempotency claims across plan/step write paths)
- `agent_execution_confirmation_ledger`
- `agent_execution_access_decision_ledger`

Compatibility note (single-step legacy shape):
- `agent_execution_ledger` / `agent_execution_current` may exist as compatibility projections.
- Once `A1` lands, they are derived from `agent_step_*` and must not be dual-write sources of truth.

Design-target indexes:
- unique plan idempotency: `(tenant_id, plan_id, idempotency_key)`
- unique step idempotency: `(tenant_id, plan_id, step_id, idempotency_key)`
- unique global idempotency: `(tenant_id, idempotency_key)`
- plan projection lookup: `(tenant_id, plan_id, updated_at)`
- step projection lookup: `(tenant_id, plan_id, step_id, updated_at)`
- replay lookup: `(tenant_id, correlation_id, created_at)`

Replay law:
- `agent_plan_current` must fully rebuild from `agent_plan_ledger`.
- `agent_step_current` must fully rebuild from `agent_step_ledger`.
- Any plan/step transition missing ledger evidence is invalid (`no silent transitions`).

## 11. Failure Modes

Agent must fail closed on:
- no simulation found
- multiple candidates unresolved
- confirmation missing
- access denied
- simulation inactive
- idempotency violation
- step timeout

Must return deterministic `reason_code`.

## 12. Rollback Model

If multi-step is enabled later:
- Each step must include `rollback_plan_ref`.
- Rollback must be simulation-based.
- No direct state mutation.

Deterministic multi-step replay rule:
- chain resume point must be derived from persisted step ledger state only
- no in-memory-only continuation authority is allowed

## 13. Acceptance Tests (Phase 1)

- `AT-AGENT-01-single-step-dispatch`
- `AT-AGENT-02-confirm-required-before-impactful-sim`
- `AT-AGENT-03-access-deny-fail-closed`
- `AT-AGENT-04-sim-not-active-fail-closed`
- `AT-AGENT-05-idempotent-retry-safe`
- `AT-AGENT-06-clarify-one-question-rule`
- `AT-AGENT-07-missing-simulation-escalates-to-dev-intake`
- `AT-AGENT-08-audit-chain-complete`
- `AT-AGENT-09-finder-top1-is-consumed-without-rerank`
- `AT-AGENT-10-multilingual-confirmation-lexicon-enforced`
- `AT-AGENT-11-idempotency-key-includes-sim-and-field-fingerprint`
- `AT-AGENT-12-missing-simulation-never-calls-simulation-executor`
- `AT-AGENT-13-current-rebuilds-from-ledger-without-drift`

## 14. CI Guardrails

Add:
- `scripts/check_agent_execution_core.sh`

Implementation note:
- `scripts/check_agent_execution_core.sh` is a design-target script; until introduced, milestone acceptance tests and readiness checks remain the source of truth.

Production lock condition:
- Execution Core is not production-locked until `scripts/check_agent_execution_core.sh` exists and passes in CI.

Fail build if:
- Agent bypasses `SimulationExecutor`.
- Agent writes business-state PH1.F tables directly outside governed execution/audit append paths.
- Agent dispatches tool as simulation.
- Missing rollback reference on impactful simulation.
- `MissingSimulationPacket` path attempts dispatch anyway.
- Finder packet is missing required proof/idempotency fields.
- Execution step lacks policy snapshot or artifact fingerprint refs.

## 14.1 Performance and Latency Discipline

World-standard latency guardrails for Agent Core:
- cache simulation-family matching artifacts from Finder output (read-only cache, TTL-bounded)
- cache thread-state projections to avoid repeated reconstruction
- prefetch only safe read-only data needed for likely next step
- avoid repeated STT calls within a single execution cycle unless explicit retry path requires it

Performance constraints are advisory to execution and must never bypass:
- confirm gates
- access gates
- ACTIVE simulation gates
- audit/idempotency requirements

### 14.2 Execution SLO Gates (production lock)

Production SLO targets (release-blocking once enabled in CI):
- `agent_dispatch_p95_ms <= 1200`
- `agent_dispatch_p99_ms <= 2500`
- `execution_error_rate <= 0.5%` (rolling 24h)
- `idempotency_duplicate_side_effect_rate = 0`

Fail-closed gate:
- If any SLO breaches threshold, release remains blocked until corrected or explicitly builder-approved with a time-bounded waiver.

## 15. Risk Controls

| Risk | Mitigation |
|---|---|
| Runaway multi-step loop | Single-step only in Phase 1 |
| Hidden auto-execution | Confirm + access required |
| Side-effect bypass | `SimulationExecutor` hard gate |
| Replay drift | Deterministic idempotency |
| Non-deterministic planning | Finder-owned deterministic top-1 + no-rerank law in AEC |
| Governance bypass | ACTIVE simulation required |

## 16. Phase Breakdown

- `M0` baseline audit
- `M1` `AgentGoalRequest` + `AgentExecutionPlan` contracts
- `M2` finder-packet consumption + deterministic single-step dispatch
- `M3` clarify + confirm enforcement
- `M4` access + ACTIVE simulation enforcement
- `M5` idempotency + audit chain
- `M6` CI guardrails
- `M7` multi-step planning (future)

## 17. Interaction With Other Plans

- Simulation Finder provides candidate simulations.
- AUTO_MICRO never triggers Agent directly.
- Builder promotions affect simulation availability (ACTIVE state).
- Learning loop feeds Finder ranking bonus only (never auto-executes in AEC).

## 17.1 Why 37 + 38 Together Are Differentiated

Combined differentiators over generic agent/chat behavior:
- deterministic, auditable execution under simulation governance
- replayable agent chains with DB-truth resume semantics
- Dev Intake capability growth loop when simulation does not exist
- gold-output learning loop improves matching without unsafe autonomy

## 17.2 Deterministic replay contract (AEC side)

Each execution decision must persist:
- `finder_packet_ref`
- `policy_snapshot_ref`
- `catalog_snapshot_ref`
- `access_decision_ref`
- `confirmation_parse_proof_ref` (when confirm required)
- `idempotency_key`
- `rollback_plan_ref` (impactful classes)

Replay invariant:
- re-running from persisted refs must produce the same dispatch/no-dispatch decision.
- if invariant fails, request must fail closed and emit deterministic replay error code.

## 18. Final Principle

Agent Execution Core:
- Does not execute directly.
- Does not mutate directly.
- Does not bypass.
- Only orchestrates simulations.
- Fully governed.
- Fully auditable.
- Fully deterministic.

## Recommendation

Implement Phase 1 (single-step agent) first.
Do not implement multi-step until single-step is stable.

## 19. Multi-step Agent Expansion (Do-it-all Tasks)

Goal:
- Enable Agent Execution Core to run multi-step plans deterministically:
  - `plan -> step ledger -> execute step-by-step via simulations -> pause/clarify/confirm -> resume -> complete`.

Hard rules:
- Still simulation-gated per step.
- Confirm required per impactful step.
- Full idempotency per step.
- Persist plan + step state in `PH1.F`.
- Replayable.

### 19.0 Canonical lifecycle states and transitions (authoritative)

Plan states (`agent_plan_current.status`):
- `PLAN_CREATED`
- `PLAN_WAITING_CONFIRM`
- `PLAN_WAITING_CLARIFY`
- `PLAN_READY`
- `PLAN_IN_PROGRESS`
- `PLAN_PAUSED`
- `PLAN_COMPLETED`
- `PLAN_FAILED`
- `PLAN_CANCELLED`

Allowed plan transitions only:
- `PLAN_CREATED -> PLAN_WAITING_CONFIRM | PLAN_WAITING_CLARIFY | PLAN_READY`
- `PLAN_WAITING_CONFIRM -> PLAN_READY | PLAN_CANCELLED`
- `PLAN_WAITING_CLARIFY -> PLAN_READY | PLAN_CANCELLED`
- `PLAN_READY -> PLAN_IN_PROGRESS`
- `PLAN_IN_PROGRESS -> PLAN_PAUSED | PLAN_COMPLETED | PLAN_FAILED`
- `PLAN_PAUSED -> PLAN_IN_PROGRESS | PLAN_CANCELLED`

Terminal plan states:
- `PLAN_COMPLETED`
- `PLAN_FAILED`
- `PLAN_CANCELLED`

Step states (`agent_step_current.status`):
- `STEP_PENDING`
- `STEP_WAITING_CONFIRM`
- `STEP_WAITING_CLARIFY`
- `STEP_READY`
- `STEP_EXECUTING`
- `STEP_SUCCEEDED`
- `STEP_FAILED_RETRYABLE`
- `STEP_FAILED_TERMINAL`
- `STEP_SKIPPED`

Allowed step transitions only:
- `STEP_PENDING -> STEP_WAITING_CONFIRM | STEP_WAITING_CLARIFY | STEP_READY`
- `STEP_WAITING_CONFIRM -> STEP_READY | STEP_FAILED_TERMINAL`
- `STEP_WAITING_CLARIFY -> STEP_READY | STEP_FAILED_TERMINAL`
- `STEP_READY -> STEP_EXECUTING`
- `STEP_EXECUTING -> STEP_SUCCEEDED | STEP_FAILED_RETRYABLE | STEP_FAILED_TERMINAL`
- `STEP_FAILED_RETRYABLE -> STEP_READY | STEP_FAILED_TERMINAL`

Terminal step states:
- `STEP_SUCCEEDED`
- `STEP_FAILED_TERMINAL`
- `STEP_SKIPPED`

No-silent-transition law:
- Every status transition above requires one append-only ledger row with `from_status`, `to_status`, and `transition_reason_code`.

### 19.1 Canonical idempotency key recipes (authoritative)

Every multi-step write path must use one deterministic idempotency recipe:
- Plan create:
  - `agent_plan_create:{tenant_id}:{correlation_id}:{finder_packet_hash}:{plan_template_version}`
- Plan transition:
  - `agent_plan_transition:{tenant_id}:{plan_id}:{from_status}:{to_status}:{event_seq}`
- Step create/materialize:
  - `agent_step_create:{tenant_id}:{plan_id}:{step_id}:{simulation_id}:{required_fields_fingerprint}`
- Step execute attempt:
  - `agent_step_execute:{tenant_id}:{plan_id}:{step_id}:{attempt_no}:{simulation_id}:{required_fields_fingerprint}:{policy_snapshot_version}`
- Step resume:
  - `agent_step_resume:{tenant_id}:{plan_id}:{step_id}:{resume_token}`
- Step complete/finalize:
  - `agent_step_finalize:{tenant_id}:{plan_id}:{step_id}:{terminal_status}:{outcome_hash}`
- Plan finalize:
  - `agent_plan_finalize:{tenant_id}:{plan_id}:{terminal_status}`

Uniqueness/index law:
- `agent_idempotency_index` must enforce uniqueness on `(tenant_id, idempotency_key)`.
- Duplicate key replay must return the original proof ref and produce no new side effect.

### Run A1 — DB truth for Agent plans + step state

Files:
- `docs/03_BUILD_LEDGER.md`
- `crates/selene_storage/src/ph1f.rs`
- `crates/selene_storage/src/repo.rs`
- `crates/selene_storage/tests/ph1_agent/db_wiring.rs` (new)
- `docs/DB_WIRING/PH1_AGENT.md` (new)
- `docs/DB_WIRING/migrations/00xx_ph1_agent_plan_tables.sql` (new, use next number)

Tables (design target):
- `agent_plan_ledger` / `agent_plan_current`
- `agent_step_ledger` / `agent_step_current`
- `agent_idempotency_index`
- `agent_execution_confirmation_ledger`
- `agent_execution_access_decision_ledger`

Tests to add:
- `at_agent_db_01_plan_roundtrip_append_only_and_idempotent`
- `at_agent_db_02_step_roundtrip_append_only_and_idempotent`
- `at_agent_db_03_replay_rebuild_equals_current_projection`
- `at_agent_db_04_step_idempotency_prevents_duplicate_commit`

Commands:
- `cargo test -p selene_storage at_agent_db_0 -- --nocapture`
- `bash scripts/check_ph1_readiness_strict.sh`
- `bash scripts/check_agent_sim_finder_core_acceptance.sh` (required once script exists; design-target pre-lock)
- `bash scripts/check_agent_execution_core.sh` (required once script exists; design-target pre-lock)

### Run A2 — Plan builder (deterministic) + no execution yet

Files:
- `crates/selene_os/src/app_ingress.rs`
- `crates/selene_engines/src/ph1simfinder.rs` (Finder output consumption)
- `crates/selene_kernel_contracts/src/ph1agent.rs` (if new plan structs need contracts)
- `docs/03_BUILD_LEDGER.md`

Behavior:
- Create `AgentExecutionPlan` with ordered steps from Finder output (initially 2-step demo).
- Persist plan to `PH1.F`.
- Return `Plan created` + next step required (`confirm`/`clarify`).
- Deterministic plan construction is AEC-owned only and uses a template-registry artifact:
  - select `plan_template_ref` by matched simulation family from Finder `SimulationMatchPacket`.
  - step ordering = template `step_ordinal` ascending.
  - tie-break = `step_template_id` lexicographic ascending.
  - compute `plan_hash = sha256(finder_packet_hash + plan_template_ref + ordered_step_fingerprints + policy_snapshot_ref)`.
  - identical inputs must yield identical `plan_hash` and step order.

Tests to add:
- `at_agent_plan_01_plan_created_persisted_and_returns_next_step`
- `at_agent_plan_02_plan_is_deterministic_for_same_inputs`
- `at_agent_plan_03_missing_sim_emits_dev_intake_no_plan`

Commands:
- `cargo test -p selene_os at_agent_plan_0 -- --nocapture`
- `bash scripts/check_ph1_readiness_strict.sh`
- `bash scripts/check_agent_sim_finder_core_acceptance.sh` (required once script exists; design-target pre-lock)
- `bash scripts/check_agent_execution_core.sh` (required once script exists; design-target pre-lock)

### Run A3 — Step executor loop (single-step execution with resume)

Files:
- `crates/selene_os/src/app_ingress.rs`
- `crates/selene_os/src/simulation_executor.rs`
- `crates/selene_storage/src/ph1f.rs`
- `docs/03_BUILD_LEDGER.md`

Behavior:
- Execute exactly one step per turn (Phase-2 safe mode).
- Update `step_current` and append `step_ledger`.
- Pause on `confirm`/`clarify`.
- Resume based on `plan_id + step_id`.
- Enforce per-step access outcomes deterministically: `ALLOW` executes, `DENY` fail-closed, `ESCALATE` fail-closed with `ACCESS_AP_REQUIRED`.
- Enforce per-step ACTIVE simulation proof before dispatch.

Tests to add:
- `at_agent_exec_01_executes_step1_persists_outcome_and_pauses`
- `at_agent_exec_02_resume_executes_step2_and_completes_plan`
- `at_agent_exec_03_confirm_required_blocks_until_yes`
- `at_agent_exec_04_step_retry_is_idempotent_no_double_side_effect`
- `at_agent_exec_05_double_resume_same_step_is_single_commit`
- `at_agent_exec_06_parallel_turn_collision_same_plan_step_fails_closed`
- `at_agent_exec_07_step_access_allow_executes`
- `at_agent_exec_08_step_access_deny_fails_closed`
- `at_agent_exec_09_step_access_escalate_fails_closed_ap_required`
- `at_agent_exec_10_step_inactive_sim_fails_closed_with_proof`

Commands:
- `cargo test -p selene_os at_agent_exec_0 -- --nocapture`
- `bash scripts/check_ph1_readiness_strict.sh`
- `bash scripts/check_agent_sim_finder_core_acceptance.sh` (required once script exists; design-target pre-lock)
- `bash scripts/check_agent_execution_core.sh` (required once script exists; design-target pre-lock)

### Run A4 — Multi-step pizza demo as missing-sim flow + later capability hook

Files:
- `crates/selene_os/src/app_ingress.rs`
- `crates/selene_storage/src/ph1f.rs`
- `docs/03_BUILD_LEDGER.md`

Behavior:
- `order pizza` produces `MissingSimulationPacket` -> Dev Intake row only when pizza capability has neither `Active` nor `Draft` simulation in catalog (must follow Finder `Active -> Draft -> None` rule).
- If pizza simulation exists in `Draft`, AEC must relay Finder `RefusePacket` (`SIM_FINDER_SIMULATION_INACTIVE`) and must not create Dev Intake row.
- When a pizza simulation exists later, plan builder can use it only if:
  - catalog status is `Active`,
  - `catalog_snapshot_ref` proves `Active` at plan-build time,
  - builder activation proof ref is present (`builder_activation_proof_ref`).

Tests to add:
- `at_agent_demo_01_order_pizza_routes_to_dev_intake`
- `at_agent_demo_02_dev_intake_dedupe_fingerprint_is_stable`
- `at_agent_demo_03_pizza_plan_build_rejected_when_sim_not_active`
- `at_agent_demo_04_pizza_plan_build_allows_only_with_active_proof`
- `at_agent_demo_05_order_pizza_draft_routes_to_refuse_not_missing_sim`

Commands:
- `cargo test -p selene_os at_agent_demo_0 -- --nocapture`
- `bash scripts/check_ph1_readiness_strict.sh`
- `bash scripts/check_agent_sim_finder_core_acceptance.sh` (required once script exists; design-target pre-lock)
- `bash scripts/check_agent_execution_core.sh` (required once script exists; design-target pre-lock)
