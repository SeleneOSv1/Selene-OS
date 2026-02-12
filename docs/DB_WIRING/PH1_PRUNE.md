# PH1_PRUNE DB Wiring (Design vNext)

## A) Engine Header
- engine_id: PH1.PRUNE
- layer: Conversation Assist
- authority: Non-Authoritative
- role: Missing-field pruning for one-question clarify discipline
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
- Invoked_by: OS step: before PH1.X clarify when multiple required fields are missing
- Inputs_from: PH1.NLP required_fields_missing, PH1.X pending clarify context
- Outputs_to: pruned_missing_field returned to Selene OS and forwarded to PH1.X
- Invocation_condition: OPTIONAL(multiple missing fields)
- Not allowed:
  - Engine-to-engine direct calls.
  - Any execution commit or authority mutation.
  - Any bypass of Selene OS orchestration.

## E) Acceptance Tests
- AT-PRUNE-01: Selene OS can invoke capability_id and output is schema-valid.
- AT-PRUNE-02: Output is bounded and deterministic ordering is preserved.
