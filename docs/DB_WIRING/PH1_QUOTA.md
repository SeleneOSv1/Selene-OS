# PH1_QUOTA DB Wiring (Design vNext)

## A) Engine Header
- engine_id: PH1.QUOTA
- layer: Enterprise Support
- authority: Authoritative (deterministic quota/budget decision only)
- role: Deterministic quota gate for runtime operation lanes (`QUOTA_POLICY_EVALUATE -> QUOTA_DECISION_COMPUTE`)
- placement: ENTERPRISE_SUPPORT (OS-internal governance/control path)

## B) Ownership
- Tables owned: NONE in current runtime slice (decision-only runtime)
- Reads:
  - bounded quota request context (`tenant_id`, optional `user_id`, optional `device_id`)
  - operation lane metadata (`STT | TTS | TOOL | SIMULATION | EXPORT`)
  - operation reference (`capability_id` or `tool_name`, when applicable)
  - deterministic budget/rate/policy guard signals from Selene OS
- Writes:
  - no direct table writes in this runtime slice
  - emits deterministic `ALLOW | WAIT | REFUSE` quota decisions only

## C) Hard Boundaries
- Deterministic decisions only (no randomness).
- PH1.QUOTA never grants authority and never changes gate order.
- `WAIT` is allowed only when policy permits; otherwise quota path must return `REFUSE`.
- PH1.QUOTA must never execute actions, tools, or simulations.
- PH1.QUOTA must never bypass Access/Authority or Simulation gating.

## D) Wiring
- Invoked_by: Selene OS before lane execution for STT/TTS/TOOL/SIMULATION/EXPORT paths.
- Inputs_from:
  - tenant/user/device scope
  - lane operation metadata (`operation_kind`, `capability_id`/`tool_name`)
  - bounded request-time snapshot (`now`, optional `cost_hint`)
  - deterministic quota guards (`rate_limit_exceeded`, `budget_exceeded`, `policy_blocked`, `wait_permitted`)
- Outputs_to:
  - `quota_policy_bundle` (`throttle_cause`, allow/wait/refuse posture, wait budget)
  - `quota_decision_bundle` (`decision`, optional `wait_ms`, `reason_code`)
- Invocation_condition: ENTERPRISE_SUPPORT (quota enforcement enabled)
- Deterministic sequence:
  - `QUOTA_POLICY_EVALUATE`:
    - validates lane scope and operation reference.
    - computes deterministic throttle cause (`NONE | RATE_LIMIT | BUDGET_EXCEEDED | POLICY_BLOCKED`).
    - computes deterministic wait/refuse posture under policy.
  - `QUOTA_DECISION_COMPUTE`:
    - emits exactly one decision (`ALLOW | WAIT | REFUSE`).
    - emits `wait_ms` only for `WAIT`.
    - enforces non-authority and no-gate-order-change invariants.
- Not allowed:
  - engine-to-engine direct calls
  - hidden fallback paths that bypass quota decision
  - non-deterministic jitter in decision output

## E) Related Engine Boundaries
- `PH1.TENANT`: tenant resolution/policy context is upstream; PH1.QUOTA consumes tenant-scoped inputs and must never guess tenant scope.
- `PH1.COST`: PH1.COST remains non-authoritative routing hints; PH1.QUOTA is the authoritative quota lane gate for `ALLOW | WAIT | REFUSE`.
- `PH1.C` / `PH1.TTS` / `PH1.E` / `PH1.SCHED` / `PH1.EXPORT`: quota decisions gate whether these operation lanes may proceed now, pause, or fail closed.
- `PH1.J`: quota decisions are reason-coded and auditable.

## F) Acceptance Tests
- AT-QUOTA-01: rate limiting is enforced deterministically.
- AT-QUOTA-02: `WAIT` vs `REFUSE` follows policy.
