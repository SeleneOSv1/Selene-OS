# PH1_CONTEXT DB Wiring (Design vNext)

## A) Engine Header
- engine_id: PH1.CONTEXT
- layer: Context Composition
- authority: Non-Authoritative
- role: Bounded context bundle assembly
- placement: ALWAYS_ON

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


## D) Wiring
- Invoked_by: OS step: before PH1.NLP and PH1.X execution
- Inputs_from: PH1.M memory bundles, PH1_MULTI hints, PH1_DOC/PH1_VISION/PH1_WEBINT evidence bundles
- Outputs_to: context_bundle returned to Selene OS and forwarded to PH1.NLP/PH1.X
- Invocation_condition: ALWAYS
- Not allowed:
  - Engine-to-engine direct calls.
  - Any execution commit or authority mutation.
  - Any bypass of Selene OS orchestration.

## E) Acceptance Tests
- AT-CONTEXT-01: Selene OS can invoke capability_id and output is schema-valid.
- AT-CONTEXT-02: Output is bounded and deterministic ordering is preserved.
