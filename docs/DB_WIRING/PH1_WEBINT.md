# PH1_WEBINT DB Wiring (Design vNext)

## A) Engine Header
- engine_id: PH1.WEBINT
- layer: Evidence Analyzer
- authority: Non-Authoritative
- role: Web evidence interpretation from tool outputs
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
- Read-only analyzer constraint: no web/tool execution; interpretation only from PH1.E evidence.

## D) Wiring
- Invoked_by: OS step: after PH1.E returns web/tool evidence
- Inputs_from: PH1.E tool_response evidence only
- Outputs_to: webint_evidence_bundle returned to Selene OS and forwarded to PH1_CONTEXT/PH1.X
- Invocation_condition: OPTIONAL(tool evidence available)
- Not allowed:
  - Engine-to-engine direct calls.
  - Any execution commit or authority mutation.
  - Any bypass of Selene OS orchestration.

## E) Acceptance Tests
- AT-WEBINT-01: Selene OS can invoke capability_id and output is schema-valid.
- AT-WEBINT-02: Output is bounded and deterministic ordering is preserved.
