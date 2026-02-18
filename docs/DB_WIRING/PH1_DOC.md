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
- No-inference rule is mandatory: PH1.DOC must fail closed when citations/snippets are not grounded in extracted evidence text.

## D) Wiring
- Invoked_by: OS step: when user-provided document evidence is attached
- Inputs_from: Selene OS document refs/blobs only
- Outputs_to: doc_evidence_bundle returned to Selene OS and forwarded to PH1_SUMMARY/PH1_CONTEXT/PH1.NLP
- Invocation_condition: OPTIONAL(document evidence present)
- Deterministic sequence:
  - DOC_EVIDENCE_EXTRACT (build bounded evidence items)
  - DOC_CITATION_MAP_BUILD (self-check validation)
  - If validation_status != OK, OS refuses/fails closed and does not forward bundle
- Not allowed:
  - Engine-to-engine direct calls.
  - Any execution commit or authority mutation.
  - Any bypass of Selene OS orchestration.

## E) Acceptance Tests
- AT-DOC-01: Selene OS can invoke capability_id and output is schema-valid.
- AT-DOC-02: Output is bounded and deterministic ordering is preserved.
- AT-DOC-03: Evidence extraction respects budget bound and truncates deterministically.
- AT-DOC-04: Citation validation fails closed for non-evidence-backed snippets.

## F) Related Engine Boundary (`PH1.CONTEXT`)
- PH1.DOC evidence bundles are advisory and may be consumed by PH1.CONTEXT only via Selene OS context composition.
- PH1.DOC does not decide context ordering; PH1.CONTEXT enforces final `CONTEXT_BUNDLE_BUILD` + `CONTEXT_BUNDLE_TRIM` ordering/validation.
