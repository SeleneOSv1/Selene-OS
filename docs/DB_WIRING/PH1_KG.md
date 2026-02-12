# PH1_KG DB Wiring (Design vNext)

## A) Engine Header
- engine_id: PH1.KG
- layer: Knowledge Assist
- authority: Non-Authoritative
- role: Entity/knowledge grounding hints
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
- Invoked_by: OS step: when entity grounding or knowledge linking is requested
- Inputs_from: Selene OS entity candidates, PH1_CONTEXT topic hints
- Outputs_to: kg_fact_bundle returned to Selene OS and forwarded to PH1_CONTEXT/PH1.NLP
- Invocation_condition: OPTIONAL(grounding requested)
- Not allowed:
  - Engine-to-engine direct calls.
  - Any execution commit or authority mutation.
  - Any bypass of Selene OS orchestration.

## E) Acceptance Tests
- AT-KG-01: Selene OS can invoke capability_id and output is schema-valid.
- AT-KG-02: Output is bounded and deterministic ordering is preserved.
