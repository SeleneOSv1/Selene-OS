# PH1_DOC DB Wiring (Design vNext)

## A) Engine Header
- engine_id: PH1.DOC
- layer: Evidence Analyzer
- authority: Non-Authoritative
- role: Document evidence extraction
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
- Read-only analyzer constraint: no web/tool execution; extraction only from OS-provided evidence.

## D) Wiring
- Invoked_by: OS step: when user-provided document evidence is attached
- Inputs_from: Selene OS document refs/blobs only
- Outputs_to: doc_evidence_bundle returned to Selene OS and forwarded to PH1_CONTEXT/PH1.NLP
- Invocation_condition: OPTIONAL(document evidence present)
- Not allowed:
  - Engine-to-engine direct calls.
  - Any execution commit or authority mutation.
  - Any bypass of Selene OS orchestration.

## E) Acceptance Tests
- AT-DOC-01: Selene OS can invoke capability_id and output is schema-valid.
- AT-DOC-02: Output is bounded and deterministic ordering is preserved.
