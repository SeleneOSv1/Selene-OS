# PH1_DIAG DB Wiring (Design vNext)

## A) Engine Header
- engine_id: PH1.DIAG
- layer: Conversation Assist
- authority: Non-Authoritative
- role: Final pre-directive diagnostic checks
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
- Invoked_by: OS step: immediately before PH1.X final directive emission
- Inputs_from: PH1.NLP draft/clarify outputs, PH1.ACCESS policy flags
- Outputs_to: diagnostic_flags returned to Selene OS and forwarded to PH1.X
- Invocation_condition: OPTIONAL(pre-finalization diagnostic pass)
- Not allowed:
  - Engine-to-engine direct calls.
  - Any execution commit or authority mutation.
  - Any bypass of Selene OS orchestration.

## E) Acceptance Tests
- AT-DIAG-01: Selene OS can invoke capability_id and output is schema-valid.
- AT-DIAG-02: Output is bounded and deterministic ordering is preserved.
