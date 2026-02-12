# PH1_RLL DB Wiring (Design vNext)

## A) Engine Header
- engine_id: PH1.RLL
- layer: Offline Learning
- authority: Non-Authoritative
- role: Offline ranking/learning loop for artifact proposals
- placement: OFFLINE_ONLY

## B) Ownership
- Tables owned: NONE (vNext design)
- Reads:
  - Structured, OS-supplied inputs from upstream engine outputs.
  - Optional evidence references (conversation/audit pointers) when Selene OS provides them.
- Writes: NONE (no direct persistence in vNext)

## C) Hard Boundaries
- Non-authoritative and non-executing; advisory output only.
- Must never grant authority, alter permissions, or bypass Access + Simulation ordering.
- Must never perform side effects, tool execution, or engine-to-engine direct calls.
- Offline-only constraint: never invoked in-turn; produces ranked artifacts only.

## D) Wiring
- Invoked_by: OS offline pipeline step: artifact ranking
- Inputs_from: PH1_PATTERN proposals, offline evaluation metrics
- Outputs_to: ranked_policy_artifacts returned to Selene OS offline artifact queue
- Invocation_condition: OFFLINE_ONLY
- Not allowed:
  - Engine-to-engine direct calls.
  - Any execution commit or authority mutation.
  - Any bypass of Selene OS orchestration.

## E) Acceptance Tests
- AT-RLL-01: Selene OS can invoke capability_id and output is schema-valid.
- AT-RLL-02: Output is bounded and deterministic ordering is preserved.
