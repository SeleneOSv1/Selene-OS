# PH1_PUZZLE DB Wiring (Design vNext)

## A) Engine Header
- engine_id: PH1.PUZZLE
- layer: Understanding Assist
- authority: Non-Authoritative
- role: Ambiguity candidate generation for hard utterances
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
- Invoked_by: OS step: after PH1.SRL when ambiguity flags are present
- Inputs_from: PH1.SRL frame, PH1.NLP ambiguity flags
- Outputs_to: disambiguation_candidates returned to Selene OS and forwarded to PH1.NLP
- Invocation_condition: OPTIONAL(ambiguity detected)
- Not allowed:
  - Engine-to-engine direct calls.
  - Any execution commit or authority mutation.
  - Any bypass of Selene OS orchestration.

## E) Acceptance Tests
- AT-PUZZLE-01: Selene OS can invoke capability_id and output is schema-valid.
- AT-PUZZLE-02: Output is bounded and deterministic ordering is preserved.
