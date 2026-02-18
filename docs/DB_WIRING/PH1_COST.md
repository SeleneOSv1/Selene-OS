# PH1_COST DB Wiring (Design vNext)

## A) Engine Header
- engine_id: PH1.COST
- layer: Planning Assist
- authority: Non-Authoritative
- role: Unified turn-policy pacing + budget guardrails for runtime routes
- placement: TURN_OPTIONAL

## B) Ownership
- Tables owned: NONE (vNext design)
- Reads:
  - Structured, OS-supplied per-user/day route budget snapshots.
  - Optional policy flags from Selene OS (budget profile/degrade policy).
  - Related route scopes for STT/LLM/TTS/tool lanes.
- Writes: NONE (no direct persistence in vNext)

## C) Hard Boundaries
- Non-authoritative and non-executing; advisory output only.
- Must never grant authority, alter permissions, or bypass Access + Simulation ordering.
- Must never perform side effects, tool execution, or engine-to-engine direct calls.
- Hard no-autonomous-action rule: urgency/delivery metadata may tune pacing/budgets only; it must never trigger execution.
- Hard no-truth-mutation rule: PH1.COST may only choose permissible routing/degrade hints (shorter response, lower retries, budget tier); it must never change user-visible truth or action outcomes.

## D) Wiring
- Invoked_by: OS step: after policy/context extraction and before route execution for PH1.C/PH1.D/PH1.TTS/PH1.E.
- Inputs_from: per-user/day budget snapshot for STT, LLM, TTS, and TOOL lanes.
- Outputs_to: `cost_guardrails` bundle returned to Selene OS and forwarded as route-budget + urgency/pacing hints to PH1.C, PH1.D, PH1.TTS, PH1.E (and pacing hinting for PH1.X phrasing length).
- Invocation_condition: OPTIONAL(cost guardrails enabled)
- Deterministic sequence:
  - COST_BUDGET_PLAN_BUILD (derive lane guardrails)
  - COST_ROUTE_GUARD_VALIDATE (self-check lane guardrails)
  - If validation_status != OK, OS refuses/fails closed and does not forward bundle.
- Not allowed:
  - Engine-to-engine direct calls.
  - Any execution commit or authority mutation.
  - Any bypass of Selene OS orchestration.

## E) Acceptance Tests
- AT-COST-01: Selene OS can invoke capability_id and output is schema-valid.
- AT-COST-02: Per-user/day lane budgets deterministically trigger degrade modes (shorter responses, lower retries, budget route tier).
- AT-COST-03: Budget exhaustion deterministically refuses that lane (no hidden retries).
- AT-COST-04: Guardrail validation drift fails closed before downstream routing.

## F) Related Engine Boundary (`PH1.QUOTA`)
- PH1.COST remains a non-authoritative planning assist that emits route/degrade hints only.
- PH1.QUOTA is the authoritative enterprise lane gate that emits deterministic `ALLOW | WAIT | REFUSE` decisions.
- Selene OS must preserve this ordering: PH1.COST hints may inform routing, but PH1.QUOTA decisions control whether a lane proceeds now, pauses, or fails closed.
