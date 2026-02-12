# PH1_LANG DB Wiring (Design vNext)

## A) Engine Header
- engine_id: PH1.LANG
- layer: Understanding Assist
- authority: Non-Authoritative
- role: Language hinting across STT and parse
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
- Invoked_by: OS step: before PH1.C retry routing and before PH1.SRL/PH1.NLP parsing
- Inputs_from: PH1.C transcript candidates, PH1.SRL semantic cues
- Outputs_to: language_hints returned to Selene OS and forwarded to PH1.C/PH1.SRL/PH1.NLP
- Invocation_condition: OPTIONAL(multilingual or language-ambiguity trigger)
- Not allowed:
  - Engine-to-engine direct calls.
  - Any execution commit or authority mutation.
  - Any bypass of Selene OS orchestration.

## E) Acceptance Tests
- AT-LANG-01: Selene OS can invoke capability_id and output is schema-valid.
- AT-LANG-02: Output is bounded and deterministic ordering is preserved.
