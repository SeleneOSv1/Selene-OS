# PH1_ENDPOINT DB Wiring (Design vNext)

## A) Engine Header
- engine_id: PH1.ENDPOINT
- layer: Perception
- authority: Non-Authoritative
- role: Endpoint boundary assist for capture/transcript alignment
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
- Invoked_by: OS step: after PH1.K frame capture and before PH1.C transcript finalization
- Inputs_from: PH1.K frame windows, PH1.C transcript timing feedback
- Outputs_to: endpoint_hints returned to Selene OS and forwarded to PH1.C/PH1.K
- Invocation_condition: OPTIONAL(capture boundary refinement)
- Not allowed:
  - Engine-to-engine direct calls.
  - Any execution commit or authority mutation.
  - Any bypass of Selene OS orchestration.

## E) Acceptance Tests
- AT-ENDPOINT-01: Selene OS can invoke capability_id and output is schema-valid.
- AT-ENDPOINT-02: Output is bounded and deterministic ordering is preserved.
