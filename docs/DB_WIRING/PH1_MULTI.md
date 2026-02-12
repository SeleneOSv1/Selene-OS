# PH1_MULTI DB Wiring (Design vNext)

## A) Engine Header
- engine_id: PH1.MULTI
- layer: Learning Assist
- authority: Non-Authoritative
- role: Multi-source hint fusion
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
- Invoked_by: OS step: pre-turn composition window
- Inputs_from: PH1_LISTEN/PH1_PAE/PH1_CACHE outputs
- Outputs_to: multi_hint_bundle returned to Selene OS and forwarded to PH1_CONTEXT/PH1.NLP
- Invocation_condition: OPTIONAL(multi-hint fusion enabled)
- Not allowed:
  - Engine-to-engine direct calls.
  - Any execution commit or authority mutation.
  - Any bypass of Selene OS orchestration.

## E) Acceptance Tests
- AT-MULTI-01: Selene OS can invoke capability_id and output is schema-valid.
- AT-MULTI-02: Output is bounded and deterministic ordering is preserved.
