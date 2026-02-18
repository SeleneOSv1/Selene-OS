# PH1_DIAG DB Wiring (Design vNext)

## A) Engine Header
- engine_id: PH1.DIAG
- layer: Conversation Assist
- authority: Non-Authoritative
- role: Final pre-directive consistency checks
- placement: TURN_OPTIONAL

## B) Ownership
- Tables owned: NONE (vNext design)
- Reads:
  - Structured, OS-supplied inputs from upstream engine outputs.
  - Intent/field/confirmation state from PH1.NLP + PH1.X flow context.
  - Privacy delivery intent flags and memory-safety flags from Selene OS.
- Writes: NONE (no direct persistence in vNext)

## C) Hard Boundaries
- Non-authoritative and non-executing; advisory output only.
- Must never grant authority, alter permissions, or bypass Access + Simulation ordering.
- Must never perform side effects, tool execution, or engine-to-engine direct calls.
- Hard non-execution rule: PH1.DIAG may only emit blocking diagnostics/reason hints; it cannot dispatch actions or mutate truth.

## D) Wiring
- Invoked_by: OS step: immediately before PH1.X final directive emission
- Inputs_from: PH1.NLP draft/clarify outputs + PH1.X pending confirmation state + policy privacy/memory flags.
- Outputs_to: `diagnostic_flags + reason_set` returned to Selene OS and forwarded to PH1.X.
- Invocation_condition: OPTIONAL(pre-finalization diagnostic pass)
- Deterministic sequence:
  - DIAG_CONSISTENCY_CHECK (derive blocking/non-blocking diagnostic flags)
  - DIAG_REASON_SET_BUILD (self-check flag consistency and emit reason_set)
  - If validation_status != OK, OS refuses/fails closed and does not forward bundle.
- Not allowed:
  - Engine-to-engine direct calls.
  - Any execution commit or authority mutation.
  - Any bypass of Selene OS orchestration.

## E) Acceptance Tests
- AT-DIAG-01: Selene OS can invoke capability_id and output is schema-valid.
- AT-DIAG-02: Output is bounded and deterministic ordering is preserved.
- AT-DIAG-03: Flag budget is enforced deterministically under multi-conflict inputs.
- AT-DIAG-04: Reason-set validation drift fails closed before PH1.X directive finalization.
