# PH1_ATTN DB Wiring (Design vNext)

## A) Engine Header
- engine_id: PH1.ATTN
- layer: Understanding Assist
- authority: Non-Authoritative
- role: Attention weighting hints for parse/context ranking
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
- Invoked_by: OS step: before PH1.NLP parse and PH1_CONTEXT assembly
- Inputs_from: PH1.C transcript spans, PH1.M memory candidate hints
- Outputs_to: attention_weights returned to Selene OS and forwarded to PH1.NLP/PH1_CONTEXT
- Invocation_condition: OPTIONAL(complex or multi-intent prompt)
- Not allowed:
  - Engine-to-engine direct calls.
  - Any execution commit or authority mutation.
  - Any bypass of Selene OS orchestration.

## E) Acceptance Tests
- AT-ATTN-01: Selene OS can invoke capability_id and output is schema-valid.
- AT-ATTN-02: Output is bounded and deterministic ordering is preserved.
