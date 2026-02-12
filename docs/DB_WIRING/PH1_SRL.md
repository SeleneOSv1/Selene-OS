# PH1_SRL DB Wiring (Design vNext)

## A) Engine Header
- engine_id: PH1.SRL
- layer: Understanding
- authority: Non-Authoritative
- role: Semantic role labeling for deterministic field scaffolding
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
- Invoked_by: OS step: immediately after PH1.C transcript_ok and before PH1.NLP
- Inputs_from: PH1.C transcript_ok payload, PH1.L session context
- Outputs_to: srl_frame returned to Selene OS and forwarded to PH1.NLP/PH1.X
- Invocation_condition: ALWAYS
- Not allowed:
  - Engine-to-engine direct calls.
  - Any execution commit or authority mutation.
  - Any bypass of Selene OS orchestration.

## E) Acceptance Tests
- AT-SRL-01: Selene OS can invoke capability_id and output is schema-valid.
- AT-SRL-02: Output is bounded and deterministic ordering is preserved.
