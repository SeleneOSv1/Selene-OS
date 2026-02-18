# PH1_VISION DB Wiring (Design vNext)

## A) Engine Header
- engine_id: PH1.VISION
- layer: Evidence Analyzer
- authority: Non-Authoritative
- role: Visual perception evidence extraction (image/screenshot/diagram)
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
- Read-only analyzer constraint: no camera/tool execution; extraction only from OS-provided evidence.
- Opt-in only: this engine runs only when explicitly enabled by policy/runtime configuration.
- No inference beyond visible content: outputs must be evidence-backed to what is present in the provided visual input.

## D) Wiring
- Invoked_by: OS step: when user-provided visual evidence is attached
- Inputs_from: Selene OS image refs/blobs only
- Outputs_to: vision_evidence_bundle returned to Selene OS and forwarded to PH1_MULTI/PH1_CONTEXT
- Invocation_condition: OPTIONAL(visual evidence present AND vision opt-in enabled)
- Not allowed:
  - Engine-to-engine direct calls.
  - Any execution commit or authority mutation.
  - Any bypass of Selene OS orchestration.

## E) Acceptance Tests
- AT-VISION-01: Selene OS can invoke capability_id and output is schema-valid.
- AT-VISION-02: Output is bounded and deterministic ordering is preserved.
- AT-VISION-03: Engine is not invoked when vision opt-in is disabled.
- AT-VISION-04: Outputs include only visible-content evidence (no inferred unseen facts).
