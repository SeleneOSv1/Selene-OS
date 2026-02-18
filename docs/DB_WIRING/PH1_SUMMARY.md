# PH1_SUMMARY DB Wiring (Design vNext)

## A) Engine Header
- engine_id: PH1.SUMMARY
- layer: Evidence Analyzer
- authority: Non-Authoritative
- role: Evidence-backed summary synthesis for downstream context/understanding
- placement: TURN_OPTIONAL

## B) Ownership
- Tables owned: NONE (vNext design)
- Reads:
  - Structured, OS-supplied evidence bundles from upstream engines (primarily PH1.DOC and optional PH1.CONTEXT inputs).
  - Optional evidence references (conversation/audit pointers) when Selene OS provides them.
- Writes: NONE (no direct persistence in vNext)

## C) Hard Boundaries
- Non-authoritative and non-executing; advisory output only.
- Must never grant authority, alter permissions, or bypass Access + Simulation ordering.
- Must never perform side effects, tool execution, or engine-to-engine direct calls.
- Read-only synthesis constraint: summary output must be strictly evidence-backed to provided input evidence.
- No inference beyond evidence: no hidden/unstated facts, no speculative conclusions.
- Deterministic output discipline: ordering and citable evidence linkage must be replay-stable.

## D) Wiring
- Invoked_by: OS step when a bounded summary is requested for document/evidence payloads
- Inputs_from: Selene OS evidence bundle (doc/other evidence items) only
- Outputs_to: summary_bundle returned to Selene OS and forwarded to PH1_CONTEXT + PH1.NLP
- Invocation_condition: OPTIONAL(summary enabled AND evidence bundle present)
- Not allowed:
  - Engine-to-engine direct calls.
  - Any execution commit or authority mutation.
  - Any bypass of Selene OS orchestration.

## E) Acceptance Tests
- AT-SUMMARY-01: Selene OS can invoke capability_id and output is schema-valid.
- AT-SUMMARY-02: Output is bounded and deterministic ordering is preserved.
- AT-SUMMARY-03: Engine is not invoked when summary feature is disabled.
- AT-SUMMARY-04: Summary citations fail closed when any bullet is not evidence-backed.

## F) Related Engine Boundary (`PH1.CONTEXT`)
- PH1.SUMMARY output may feed PH1.CONTEXT only as advisory evidence-backed summary bullets.
- PH1.SUMMARY does not own final context ordering or trimming; PH1.CONTEXT enforces bundle-order validation.
