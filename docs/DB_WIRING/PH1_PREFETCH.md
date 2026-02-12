# PH1_PREFETCH DB Wiring (Design vNext)

## A) Engine Header
- engine_id: PH1.PREFETCH
- layer: Planning Assist
- authority: Non-Authoritative
- role: Prefetch candidate suggestion
- placement: TURN_OPTIONAL

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
- Invoked_by: OS step: after PH1.NLP intent classification and before idle windows
- Inputs_from: PH1.NLP intent_draft, PH1.SEARCH planning hints
- Outputs_to: prefetch_candidates returned to Selene OS and forwarded to PH1.E scheduler
- Invocation_condition: OPTIONAL(prefetch policy enabled)
- Not allowed:
  - Engine-to-engine direct calls.
  - Any execution commit or authority mutation.
  - Any bypass of Selene OS orchestration.

## E) Acceptance Tests
- AT-PREFETCH-01: Selene OS can invoke capability_id and output is schema-valid.
- AT-PREFETCH-02: Output is bounded and deterministic ordering is preserved.
