# PH1_MULTI DB Wiring (Design vNext)

## A) Engine Header
- engine_id: PH1.MULTI
- layer: Learning Assist
- authority: Non-Authoritative
- role: Multimodal context fusion (voice/text first, vision/doc when provided)
- placement: TURN_OPTIONAL

## B) Ownership
- Tables owned: NONE (vNext design)
- Reads:
  - Structured, OS-supplied inputs from upstream outputs (`PH1.LISTEN`, `PH1.PAE`, `PH1.CACHE`).
  - Optional evidence bundles from `PH1.VISION` / `PH1.DOC` when present.
- Writes: NONE (no direct persistence in vNext)

## C) Hard Boundaries
- Non-authoritative and non-executing; advisory bundle output only.
- Must never grant authority, alter permissions, or bypass Access + Simulation ordering.
- Must never perform side effects, tool execution, or engine-to-engine direct calls.
- Privacy-scoped rule: all fused signals must be privacy-scoped; otherwise fail closed.
- Evidence-backed rule: vision/document signals must carry evidence references; otherwise fail closed.

## D) Wiring
- Invoked_by: OS pre-turn composition window.
- Inputs_from: `PH1.LISTEN` (`LISTEN_SIGNAL_FILTER=OK` output only) + `PH1.PAE` + `PH1.CACHE` outputs, with optional `PH1.VISION`/`PH1.DOC` evidence bundles.
- Outputs_to: `multi_hint_bundle` returned to Selene OS, then forwarded to `PH1.CONTEXT` (and bounded hints to `PH1.NLP` when enabled by OS policy).
- Invocation_condition: OPTIONAL (multi-fusion enabled).
- Not allowed:
  - Engine-to-engine direct calls.
  - Any execution commit or authority mutation.
  - Any bypass of Selene OS orchestration.

## E) Deterministic Capability Sequence
- Step 1: `MULTI_BUNDLE_COMPOSE`
  - Produces one selected signal + ordered bundle items under bounded budgets.
- Step 2: `MULTI_SIGNAL_ALIGN`
  - Validates selected-vs-ordered integrity, rank ordering, and evidence/diagnostic constraints.
- Hard rule: if Step 2 returns `FAIL`, Selene OS must fail closed and not forward the bundle.

## F) Acceptance Tests
- AT-MULTI-01: Selene OS can invoke `MULTI_BUNDLE_COMPOSE` and output is schema-valid.
- AT-MULTI-02: Output is bounded and deterministic ordering is preserved.
- AT-MULTI-03: Vision evidence bundles (when present) are fused with preserved evidence refs.
- AT-MULTI-04: Privacy/evidence drift fails closed (`MULTI_SIGNAL_ALIGN=FAIL` -> no forward bundle).

## G) Related Engine Boundary (`PH1.CONTEXT`)
- PH1.MULTI output may be forwarded into PH1.CONTEXT only through Selene OS and only when `MULTI_SIGNAL_ALIGN=OK`.
- PH1.MULTI never bypasses PH1.CONTEXT trim validation; PH1.CONTEXT remains responsible for final bounded context assembly checks.

## H) Related Engine Boundary (`PH1.LISTEN`)
- PH1.MULTI must consume LISTEN context only from Selene OS-validated `LISTEN_SIGNAL_FILTER=OK` bundles.
- LISTEN-derived environment hints remain advisory metadata and must not alter semantic meaning.
