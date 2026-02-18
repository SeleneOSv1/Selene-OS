# PH1_CACHE DB Wiring (Design vNext)

## A) Engine Header
- engine_id: PH1.CACHE
- layer: Learning Assist
- authority: Non-Authoritative
- role: Cached decision-path skeleton build + refresh validation
- placement: TURN_OPTIONAL

## B) Ownership
- Tables owned: NONE (vNext runtime design)
- Reads:
  - Structured, OS-supplied `intent_type`, `environment_profile_ref`, and persona snapshot refs.
  - Optional governed policy-pack refs produced by offline artifact pipelines.
- Writes: NONE (no direct persistence in vNext runtime wiring)

## C) Hard Boundaries
- Non-authoritative and non-executing; output is advisory cache skeleton metadata only.
- Must never grant authority, alter permissions, or bypass Access + Simulation ordering.
- Must never perform side effects, tool execution, or engine-to-engine direct calls.
- Hard rule: every emitted skeleton must explicitly keep `requires_access_gate=true` and `requires_simulation_gate=true`.
- Hard rule: ungoverned artifacts fail closed (`contains_ungoverned_artifacts=true` => refresh validation FAIL).

## D) Wiring
- Invoked_by: Selene OS optional planning-assist phase.
- Inputs_from:
  - PH1.NLP/PH1.X intent labels (`intent_type`).
  - PH1.LISTEN/PH1.PAE environment/policy hints (`environment_profile_ref`, route hint).
  - PH1.PERSONA snapshot hints (`persona_profile_ref`).
- Outputs_to:
  - `cache_hint_snapshot_bundle` returned to Selene OS.
  - Optional downstream consumers (PH1.PREFETCH / PH1.CONTEXT) through OS-only forwarding.
- Invocation_condition: `TURN_OPTIONAL` (cache policy enabled)
- Deterministic sequence:
  - `CACHE_HINT_SNAPSHOT_READ` builds bounded, ordered cache skeletons.
  - `CACHE_HINT_SNAPSHOT_REFRESH` self-validates selected id/order/payload integrity + governed artifact state.
  - If `validation_status != OK`, Selene OS fails closed and does not forward cache hints.
- Not allowed:
  - Engine-to-engine direct calls.
  - Any execution commit or authority mutation.
  - Any bypass of Selene OS orchestration.

## E) Related Engine Boundaries
- PH1.PAE may feed route/cost hints, but PH1.CACHE consumes only governance-approved ACTIVE artifacts.
- PH1.PAE hints are admissible only from validated `PAE_ADAPTATION_HINT_EMIT` outputs (`validation_status=OK`).
- PH1.PREFETCH may consume cache hints only as read-only candidate ordering metadata.
- PH1.CONTEXT may consume cache hints only as advisory ranking input; evidence-backed context sources remain canonical.
- PH1.CONTEXT consumption path requires `CACHE_HINT_SNAPSHOT_REFRESH=OK` before cache hints can participate in context assembly.

## F) Acceptance Tests
- AT-CACHE-01: Selene OS can invoke `CACHE_HINT_SNAPSHOT_READ` and output is schema-valid.
- AT-CACHE-02: Snapshot ordering is bounded and deterministic for identical inputs.
- AT-CACHE-03: Skeleton budget overflow fails closed deterministically.
- AT-CACHE-04: Snapshot refresh drift or ungoverned artifact flags fail closed before downstream forwarding.
