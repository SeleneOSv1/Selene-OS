# PH1_CONTEXT DB Wiring (Design vNext)

## A) Engine Header
- engine_id: PH1.CONTEXT
- layer: Context Composition
- authority: Non-Authoritative
- role: Bounded context bundle assembly + trim validation
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
- Evidence discipline: vision-derived context must carry visible-content evidence only and remain bounded.
- Web evidence discipline: when PH1.SEARCH evidence bundle is present, source-ranked order and URL provenance must be preserved in composed context.
- Multi-fusion discipline: when PH1.MULTI bundle is present, PH1.CONTEXT must preserve selected-signal + ordered-item integrity and reject non-privacy-scoped bundles.
- RLL scoring discipline: PH1.CONTEXT retrieval weighting hints derived from PH1.RLL are allowed only after governance-approved artifact activation.


## D) Wiring
- Invoked_by: OS step: before PH1.NLP and PH1.X execution
- Inputs_from: PH1.M memory bundles, PH1_MULTI hints, PH1_DOC/PH1_SUMMARY/PH1_VISION/PH1.SEARCH evidence bundles, optional PH1.CACHE advisory skeleton hints, optional PH1.KG fact bundles
- Outputs_to: context_bundle returned to Selene OS and forwarded to PH1.NLP/PH1.X
- Invocation_condition: ALWAYS
- Deterministic sequence:
  - `CONTEXT_BUNDLE_BUILD` builds selected + ordered bounded context bundle items.
  - `CONTEXT_BUNDLE_TRIM` validates selected/order integrity + related-engine alignment flags (`multi_signal_align_ok`, `cache_hint_refresh_ok`).
  - If `validation_status != OK`, Selene OS fails closed and does not forward context bundle.
- Not allowed:
  - Engine-to-engine direct calls.
  - Any execution commit or authority mutation.
  - Any bypass of Selene OS orchestration.

## E) Acceptance Tests
- AT-CONTEXT-01: Selene OS can invoke capability_id and output is schema-valid.
- AT-CONTEXT-02: Output is bounded and deterministic ordering is preserved.
- AT-CONTEXT-03: Vision evidence integration preserves evidence refs and bounded context behavior.
- AT-CONTEXT-04: Summary bundles preserve citation refs and bounded context behavior.
- AT-CONTEXT-05: Search evidence bundles preserve source-ranked order + URL provenance through context composition.
- AT-CONTEXT-06: Multi/cache alignment drift fails closed before PH1.NLP/PH1.X handoff.
- AT-CONTEXT-07: KG bundle integration preserves tenant scope + evidence refs and fails closed on cross-tenant drift.

## F) Related Engine Boundary (salience ranking)
- Salience/focus ranking for context ordering is handled inside PH1.CONTEXT deterministic composition flow.
- PH1.CONTEXT must keep ranking metadata non-authoritative; primary evidence bundles remain canonical context truth.

## G) Related Engine Boundary (`PH1.MULTI`)
- PH1.CONTEXT may consume PH1.MULTI output only when `MULTI_SIGNAL_ALIGN` returns `OK`.
- PH1.CONTEXT must preserve evidence refs for vision/document multimodal items and fail closed on missing evidence refs.

## H) Related Engine Boundary (`PH1.RLL`)
- PH1.CONTEXT must not consume raw PH1.RLL recommendations directly.
- PH1.CONTEXT may consume only governance-approved ACTIVE artifacts derived from PH1.RLL context-retrieval scoring proposals.

## I) Related Engine Boundary (`PH1.CACHE`)
- PH1.CONTEXT may consume PH1.CACHE hints only as ranking metadata for context assembly.
- PH1.CONTEXT must not treat cache hints as evidence truth; primary evidence bundles remain canonical.

## J) Related Engine Boundary (`PH1.KG`)
- PH1.CONTEXT may consume PH1.KG output only when `KG_FACT_BUNDLE_SELECT` returns `OK`.
- PH1.CONTEXT must fail closed if any KG fact candidate violates tenant scope or omits `evidence_ref`.
