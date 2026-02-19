# PH1_OS ECM (Design vNext)

## Engine Header
- engine_id: PH1.OS
- role: deterministic orchestration gate for one-turn next-move and dispatch legality
- placement: ENTERPRISE_SUPPORT

## Capability List

### capability_id: OS_POLICY_EVALUATE
- input_schema:
  - bounded envelope (`correlation_id`, `turn_id`, `max_guard_failures`, `max_diagnostics`)
  - gate posture:
    - session/understanding/confirmation (`session_active`, `transcript_ok`, `nlp_confidence_high`, `requires_confirmation`, `confirmation_received`)
    - execution dependencies (`access_allowed`, `blueprint_active`, `simulation_active`, `idempotency_required`, `idempotency_key_present`, `lease_required`, `lease_valid`)
    - dispatch request flags (`tool_requested`, `simulation_requested`)
  - invariants (`no_engine_to_engine_calls=true`, `no_simulation_no_execution=true`, `one_turn_one_move=true`)
  - optional-budget posture (per-turn, deterministic):
    - `optional_budget_enforced` (must be true)
    - `optional_invocations_requested`
    - `optional_invocations_budget`
    - `optional_invocations_skipped_budget` (must equal `requested.saturating_sub(budget)`)
    - `optional_latency_budget_ms`
    - `optional_latency_estimated_ms` (must be <= `optional_latency_budget_ms`)
- output_schema:
  - deterministic gate results (`*_gate_ok`)
  - `gate_u3_optional_budget_enforced` + echoed optional-budget fields
  - dispatch legality (`tool_dispatch_allowed`, `simulation_dispatch_allowed`, `execution_allowed`)
  - bounded `guard_failures` list + `reason_code`
  - invariants preserved (`no_engine_to_engine_calls=true`, `no_simulation_no_execution=true`, `one_turn_one_move=true`)
- allowed_callers: SELENE_OS_ONLY
- side_effects: NONE
- failure_modes: INPUT_SCHEMA_INVALID, UPSTREAM_INPUT_MISSING, BUDGET_EXCEEDED, INTERNAL_PIPELINE_ERROR
- reason_codes:
  - PH1_OS_OK_POLICY_EVALUATE
  - OS_FAIL_SESSION_GATE
  - OS_FAIL_UNDERSTANDING_GATE
  - OS_FAIL_CONFIRMATION_GATE
  - OS_FAIL_ACCESS_GATE
  - OS_FAIL_BLUEPRINT_GATE
  - OS_FAIL_SIMULATION_GATE
  - OS_FAIL_IDEMPOTENCY_GATE
  - OS_FAIL_LEASE_GATE
  - PH1_OS_INPUT_SCHEMA_INVALID
  - PH1_OS_UPSTREAM_INPUT_MISSING
  - PH1_OS_BUDGET_EXCEEDED
  - PH1_OS_INTERNAL_PIPELINE_ERROR

### capability_id: OS_DECISION_COMPUTE
- input_schema:
  - bounded envelope (`correlation_id`, `turn_id`, `max_guard_failures`, `max_diagnostics`)
  - normalized policy posture from `OS_POLICY_EVALUATE`
  - requested move posture (`chat_requested`, `clarify_required`, `clarify_owner_engine_id`, `confirm_required`, `explain_requested`, `wait_required`, `tool_requested`, `simulation_requested`)
- output_schema:
  - exactly one next move (`RESPOND | CLARIFY | CONFIRM | DISPATCH_TOOL | DISPATCH_SIMULATION | EXPLAIN | WAIT | REFUSE`)
  - fail-closed posture (`fail_closed`) and dispatch/execution booleans
  - deterministic `reason_code`
  - invariants preserved (`no_engine_to_engine_calls=true`, `no_simulation_no_execution=true`, `one_turn_one_move=true`)
- allowed_callers: SELENE_OS_ONLY
- side_effects: NONE
- failure_modes: INPUT_SCHEMA_INVALID, ONE_TURN_ONE_MOVE_CONFLICT, INTERNAL_PIPELINE_ERROR
- reason_codes:
  - PH1_OS_OK_DECISION_COMPUTE
  - OS_FAIL_ONE_TURN_ONE_MOVE
  - OS_FAIL_SESSION_GATE
  - OS_FAIL_UNDERSTANDING_GATE
  - OS_FAIL_CONFIRMATION_GATE
  - OS_FAIL_ACCESS_GATE
  - OS_FAIL_BLUEPRINT_GATE
  - OS_FAIL_SIMULATION_GATE
  - OS_FAIL_IDEMPOTENCY_GATE
  - OS_FAIL_LEASE_GATE
  - PH1_OS_INPUT_SCHEMA_INVALID
  - PH1_OS_BUDGET_EXCEEDED
  - PH1_OS_INTERNAL_PIPELINE_ERROR

## Top-Level Turn Slice (Runtime Boundary)
- PH1.OS owns one canonical top-level turn orchestration slice in `selene_os` runtime.
- The slice validates path-specific ALWAYS_ON order before invoking `OS_POLICY_EVALUATE -> OS_DECISION_COMPUTE`:
  - voice: `PH1.K -> PH1.W -> PH1.VOICE.ID -> PH1.C -> PH1.SRL -> PH1.NLP -> PH1.CONTEXT -> PH1.POLICY -> PH1.X`
  - text: `PH1.NLP -> PH1.CONTEXT -> PH1.POLICY -> PH1.X`
- TURN_OPTIONAL invocation planning is centralized in this slice:
  - requested optional engines are validated against one canonical TURN_OPTIONAL list.
  - unknown optional engine ids or sequence drift are rejected fail-closed.
  - optional invocation budget is deterministic and bounded.
- GATE-U3 rule:
  - budget posture drift (invalid optional-budget fields or latency estimate over budget) must fail closed.
  - deterministic budget breach handling is skip/degrade (optional tier), never fail-open execution.
- Clarify owner rule:
  - if `clarify_required=true`, `clarify_owner_engine_id` must equal `PH1.NLP`.
  - if `clarify_required=false`, `clarify_owner_engine_id` must be absent.
- Optional understanding-assist policy rule:
  - `PH1.PRUNE` is allowed only when `clarify_required=true`.
  - `PH1.DIAG` is allowed only when one of `clarify_required`, `confirm_required`, `tool_requested`, or `simulation_requested` is true.
  - violations are fail-closed and must not proceed to move dispatch.
- GATE-U4/GATE-U5 utility review rule (batch/daily, machine-only):
  - PH1.OS computes TURN_OPTIONAL engine utility metrics from outcome-utilization rows:
    - `decision_delta_rate`
    - `queue_learn_conversion_rate`
    - `no_value_rate`
    - `latency_cost_p95_ms`
    - `latency_cost_p99_ms`
  - deterministic action contract:
    - `KEEP` when utility thresholds pass (`GATE-U4`)
    - `DEGRADE` on first/short fail streak
    - `DISABLE_CANDIDATE` when fail streak reaches 7 days (`GATE-U5`)
  - utility review remains non-authoritative for execution: it tunes optional engine posture only and never bypasses access/simulation gates.
- Runtime-boundary rule (OFFLINE/control-plane guard):
  - live turn wiring must reject OFFLINE_ONLY engine ids (`PH1.PATTERN`, `PH1.RLL`) in runtime paths.
  - live turn wiring must reject control-plane engine ids (`PH1.GOV`, `PH1.EXPORT`, `PH1.KMS`) in runtime paths.
  - fail action is deterministic fail-closed refusal with a runtime-boundary violation reason-code; no fallback to execution is allowed.
- Delivery ownership rule (cross-engine side effects):
  - `PH1.LINK` may not execute legacy link-delivery simulation ids (`LINK_INVITE_SEND_COMMIT`, `LINK_INVITE_RESEND_COMMIT`, `LINK_DELIVERY_FAILURE_HANDLING_COMMIT`); these are fail-closed `LEGACY_DO_NOT_WIRE`.
  - `PH1.BCAST.001` is the only lifecycle owner for outbound recipient send/resend/escalation state.
  - `PH1.DELIVERY` executes provider attempts only under Selene OS simulation orchestration and returns proofs back to `PH1.BCAST.001` state flow.
  - `PH1.REM.001` remains timing-only; SMS setup remains a pre-send gate (`sms_app_setup_complete=true`) and never sends.
- This slice does not create a second authority path; final move authority remains `OS_DECISION_COMPUTE`.

## Constraints
- Engines never call engines directly; Selene OS orchestrates all sequencing.
- No Simulation -> No Execution is mandatory for simulation dispatch paths.
- One-turn-one-move is mandatory; multi-move posture must fail closed.
- PH1.OS does not execute tools or simulations directly; it emits legality only.
- PH1.OS remains deterministic for identical input snapshots.

## Related Engine Boundaries
- `PH1.X`: intent/move origin; PH1.OS validates and finalizes one deterministic move.
- `PH1.E`: tool dispatch consumes `DISPATCH_TOOL` posture only; PH1.OS does not run tools.
- `PH1.ACCESS.001/PH2.ACCESS.002`: authority truth remains external and is consumed as gate input.
- `PH1.WORK`, `PH1.LEASE`, `PH1.SCHED`: execution dependency outcomes remain external and are consumed as gate inputs.
- `PH1.J`: orchestration policy and decision outputs must remain reason-coded for audit.
