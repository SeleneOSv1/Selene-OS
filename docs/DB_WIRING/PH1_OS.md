# PH1_OS DB Wiring (Design vNext)

## A) Engine Header
- engine_id: PH1.OS
- layer: Enterprise Support
- authority: Authoritative (orchestration gate decision only)
- role: Deterministic Selene OS orchestration gate for one-turn move selection and dispatch legality (`OS_POLICY_EVALUATE -> OS_DECISION_COMPUTE`)
- placement: ENTERPRISE_SUPPORT (OS-internal control boundary before any dispatch)

## B) Ownership
- Tables owned: NONE in this runtime slice (decision-only runtime).
- Reads:
  - bounded turn gate posture (`session_active`, `transcript_ok`, `nlp_confidence_high`, `requires_confirmation`, `confirmation_received`)
  - execution gate posture (`access_allowed`, `blueprint_active`, `simulation_active`, idempotency and lease flags)
  - move request posture (`chat_requested`, `clarify_required`, `clarify_owner_engine_id`, `confirm_required`, `tool_requested`, `simulation_requested`, `wait_required`, `explain_requested`)
- Writes:
  - no direct table writes in this runtime slice.
  - emits deterministic orchestration outputs only (`policy_evaluate` + `decision_compute` bundles).

## C) Hard Boundaries
- One turn, one move: PH1.OS must emit exactly one next move.
- No Simulation -> No Execution: simulation dispatch is forbidden unless all required gates pass.
- One clarify owner: if `clarify_required=true`, `clarify_owner_engine_id` must be `PH1.NLP`; otherwise `clarify_owner_engine_id` must be omitted.
- Engines never call engines directly: PH1.OS only emits dispatch legality; it never performs engine-to-engine calls itself.
- PH1.OS must fail closed on gate failures, schema drift, and ambiguous move requests.
- PH1.OS must never execute side effects directly; execution remains simulation-gated in Selene OS.
- Outbound delivery ownership is strict and fail-closed:
  - `PH1.LINK` owns token/draft lifecycle only.
  - `PH1.BCAST.001` owns send/resend/escalation recipient lifecycle state.
  - `PH1.DELIVERY` owns provider-attempt execution/proof only.
  - `PH1.REM.001` owns timing mechanics only.
  - SMS setup remains a pre-send gate (`sms_app_setup_complete=true`) and never grants send authority by itself.
  - Legacy LINK delivery simulation ids (`LINK_INVITE_SEND_COMMIT`, `LINK_INVITE_RESEND_COMMIT`, `LINK_DELIVERY_FAILURE_HANDLING_COMMIT`) must fail closed as `LEGACY_DO_NOT_WIRE`.

## D) Wiring
- Invoked_by: Selene OS before final move dispatch each turn.
- Inputs_from:
  - `PH1.L` session state
  - `PH1.C` transcript gate outcome
  - `PH1.NLP` understanding confidence posture
  - `PH1.X` requested move posture
  - Access/Blueprint/Simulation/Idempotency/Lease gate posture from orchestration context
- Outputs_to:
  - `os_policy_bundle`:
    - per-gate booleans (`session_gate_ok`, `understanding_gate_ok`, `confirmation_gate_ok`, `access_gate_ok`, `blueprint_gate_ok`, `simulation_gate_ok`, `idempotency_gate_ok`, `lease_gate_ok`)
    - optional-budget gate posture (`gate_u3_optional_budget_enforced`, `optional_invocations_requested`, `optional_invocations_budget`, `optional_invocations_skipped_budget`, `optional_latency_budget_ms`, `optional_latency_estimated_ms`)
    - dispatch legality (`tool_dispatch_allowed`, `simulation_dispatch_allowed`, `execution_allowed`)
    - bounded `guard_failures` + `reason_code`
  - `os_decision_bundle`:
    - one `next_move` (`RESPOND | CLARIFY | CONFIRM | DISPATCH_TOOL | DISPATCH_SIMULATION | EXPLAIN | WAIT | REFUSE`)
    - `fail_closed` + dispatch/execution flags + `reason_code`
- Invocation_condition: ENTERPRISE_SUPPORT (always enabled in orchestration runtime path when PH1.OS wiring is configured on)
- Deterministic sequence:
  - top-level turn slice (canonical boundary):
    - voice path ALWAYS_ON order lock:
      - `PH1.K -> PH1.W -> PH1.VOICE.ID -> PH1.C -> PH1.SRL -> PH1.NLP -> PH1.CONTEXT -> PH1.POLICY -> PH1.X`
    - text path ALWAYS_ON order lock:
      - `PH1.NLP -> PH1.CONTEXT -> PH1.POLICY -> PH1.X` (typed transcript wrapper is upstream and must produce transcript_ok-equivalent posture)
    - TURN_OPTIONAL ordering:
      - optional engines are selected/invoked from one control point in canonical order only (no per-engine local ordering branches).
      - if requested optional ordering does not match canonical ordering semantics, PH1.OS refuses fail-closed.
      - OFFLINE/control-plane boundary guard is mandatory in live turn wiring:
        - OFFLINE_ONLY engines (`PH1.PATTERN`, `PH1.RLL`) are never invokable in runtime turn slices.
        - control-plane engines (`PH1.GOV`, `PH1.EXPORT`, `PH1.KMS`) are never invokable in runtime turn slices.
        - any appearance of these engine ids in runtime `ALWAYS_ON` or requested `TURN_OPTIONAL` sets causes deterministic fail-closed refusal.
      - per-turn optional budget contract is explicit:
        - `optional_invocations_requested`
        - `optional_invocations_budget`
        - `optional_invocations_skipped_budget` (`requested - budget`, saturating)
        - `optional_latency_budget_ms`
        - `optional_latency_estimated_ms` (bounded estimate for invoked set)
      - hard rule: if optional budget posture is malformed or estimated latency exceeds budget, PH1.OS fails closed on budget-policy drift.
      - optional understanding-assist policy bounds are deterministic and fail-closed:
        - `PH1.PRUNE` requires `clarify_required=true`.
        - `PH1.DIAG` requires one of `clarify_required`, `confirm_required`, `tool_requested`, or `simulation_requested`.
      - optional utility review (batch/daily, machine-only):
        - PH1.OS computes per-engine utility from outcome-utilization entries for TURN_OPTIONAL engines only.
        - deterministic thresholds enforce `GATE-U4`/`GATE-U5`:
          - `decision_delta_rate >= 0.08` OR `queue_learn_conversion_rate >= 0.20`
          - `no_value_rate <= 0.60`
          - `latency_cost_p95_ms <= 20`
          - `latency_cost_p99_ms <= 40`
        - policy action is deterministic:
          - `KEEP` when `GATE-U4` passes
          - `DEGRADE` when `GATE-U4` fails and fail streak < 7 days
          - `DISABLE_CANDIDATE` when `GATE-U4` fails for >= 7 consecutive days (`GATE-U5`)
  - `OS_POLICY_EVALUATE`:
    - computes gate status with deterministic precedence.
    - enforces `No Simulation -> No Execution` and bounded guard-failure budget.
  - `OS_DECISION_COMPUTE`:
    - enforces one-turn-one-move.
    - chooses exactly one move with fail-closed posture when any blocking gate fails.
    - allows `DISPATCH_SIMULATION` only when `execution_allowed=true`.
- Not allowed:
  - tool/simulation dual-dispatch in one turn
  - silent gate bypass
  - direct side-effect execution

## E) Related Engine Boundaries
- `PH1.X`: provides requested move intent; PH1.OS enforces one-turn-one-move and dispatch legality.
- `PH1.E`: read-only tool dispatch is allowed only through `DISPATCH_TOOL` posture from PH1.OS.
- `PH1.WORK` + `PH1.LEASE` + `PH1.SCHED`: execution lane dependencies are consumed as gate posture; PH1.OS does not replace their ownership.
- `PH1.ACCESS.001/PH2.ACCESS.002`: access gate remains external; PH1.OS consumes access posture only.
- `PH1.J`: policy/decision outputs must be reason-coded for audit lineage.

## F) Acceptance Tests
- AT-OS-01: No Simulation -> No Execution (simulation requested with inactive simulation gate fails closed).
- AT-OS-02: one-turn-one-move conflict fails closed (multi-move request in one turn).
- AT-OS-03: `DISPATCH_SIMULATION` emitted only when all required gates pass.
- AT-OS-04: read-only `DISPATCH_TOOL` path remains non-executing and simulation-free.
- AT-OS-05: OS wiring disabled returns deterministic `NotInvokedDisabled`.
- AT-OS-06: wiring forwards deterministic policy+decision bundles with correlation integrity.
- AT-OS-07: top-level voice path enforces canonical ALWAYS_ON sequence and rejects sequence drift fail-closed.
- AT-OS-08: top-level text path enforces canonical ALWAYS_ON sequence and forwards deterministic next move.
- AT-OS-09: top-level TURN_OPTIONAL ordering is centralized; unknown optional engine ids are rejected fail-closed.
- AT-OS-10: optional latency/invocation budget breach is rejected fail-closed at top-level orchestration boundary (GATE-U3 contract enforcement).
- AT-OS-11: optional utility review applies `GATE-U4` thresholds deterministically and emits `KEEP` only when utility + latency thresholds pass.
- AT-OS-12: sustained low utility (`GATE-U4` fail streak >= 7 days) deterministically emits `DISABLE_CANDIDATE` (`GATE-U5`).
- AT-OS-13: runtime leak attempt with OFFLINE_ONLY engine id (`PH1.PATTERN` or `PH1.RLL`) in turn orchestration input is rejected fail-closed.
- AT-OS-14: runtime leak attempt with control-plane engine id (`PH1.GOV` or `PH1.EXPORT` or `PH1.KMS`) in turn orchestration input is rejected fail-closed.
- AT-OS-15: legacy LINK delivery simulation ids are rejected fail-closed in runtime (`LEGACY_DO_NOT_WIRE`) and cannot execute through `PH1.LINK`.
- AT-OS-16: delivery ownership drift fails guardrails (`PH1.BCAST.001` lifecycle owner, `PH1.DELIVERY` provider attempts, `PH1.REM.001` timing-only, SMS setup pre-send gate enforced).
- AT-OS-17: clarify owner precedence is fail-closed (`clarify_required=true` requires `clarify_owner_engine_id=PH1.NLP`).
- AT-OS-18: optional understanding-assist policy blocks invalid clarify-loop requests (`PH1.PRUNE`/`PH1.DIAG`) when required posture flags are missing.

## G) FDX Wiring Lock (Section 5F)
- PH1.OS wiring must enforce end-to-end duplex ordering and fail-closed policy for missing/invalid upstream duplex signals.
- PH1.OS must explicitly block execution side effects from speculative outputs.
- PH1.OS must ensure FDX metric proof collection is complete before release gate closure:
  - false interrupt rate
  - missed interrupt rate
  - barge-in detect->cancel latency
  - partial transcript first-chunk latency
  - non-lexical trigger acceptance (must remain 0.0%)
