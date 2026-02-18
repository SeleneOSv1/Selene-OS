# PH1_KG DB Wiring (Design vNext)

## A) Engine Header
- engine_id: PH1.KG
- layer: Knowledge Assist
- authority: Non-Authoritative
- role: Tenant-scoped relationship grounding hints
- placement: TURN_OPTIONAL

## B) Ownership
- Tables owned: NONE (vNext runtime design)
- Reads:
  - Structured, OS-supplied entity candidates and relation hints.
  - Optional context/topic anchors from PH1.CONTEXT.
  - Optional tenant dictionary/pack references from PH1.KNOW artifacts (through Selene OS only).
- Writes: NONE (no direct persistence in vNext runtime wiring)

## C) Hard Boundaries
- Non-authoritative and non-executing; output is advisory relationship grounding only.
- Must never grant authority, alter permissions, or bypass Access + Simulation ordering.
- Must never perform side effects, tool execution, or engine-to-engine direct calls.
- Hard rule: tenant scope is mandatory; cross-tenant relation composition must fail closed.
- Hard rule: all emitted fact candidates must be evidence-backed; no guess-only facts.
- Hard rule: PH1.KG must never invent unseen entities or relationships.

## D) Wiring
- Invoked_by: Selene OS when relationship grounding/disambiguation is requested.
- Inputs_from:
  - Entity candidates + relation-type hints from Selene OS.
  - Optional context/topic hints from PH1.CONTEXT.
- Outputs_to:
  - `kg_fact_bundle` returned to Selene OS.
  - Selene OS may forward bounded grounding hints to PH1.CONTEXT and PH1.NLP.
- Invocation_condition: OPTIONAL (grounding requested)
- Deterministic sequence:
  - `KG_ENTITY_LINK`:
    - Builds selected + ordered fact candidates from relation-compatible entity pairs.
    - Enforces tenant scope + evidence presence.
  - `KG_FACT_BUNDLE_SELECT`:
    - Validates selected-vs-ordered integrity, tenant-scope preservation, and no-guessing guard.
  - If `validation_status != OK`, Selene OS fails closed and does not forward KG output.
- Not allowed:
  - Engine-to-engine direct calls.
  - Any execution commit or authority mutation.
  - Any bypass of Selene OS orchestration.

## E) Related Engine Boundaries
- PH1.CONTEXT may consume KG output only after `KG_FACT_BUNDLE_SELECT=OK`.
- PH1.NLP may consume KG hints as disambiguation metadata only; KG output never overrides transcript evidence.
- PH1.KNOW remains the dictionary/pack source (`docs/DB_WIRING/PH1_KNOW.md` + `docs/ECM/PH1_KNOW.md`); PH1.KG relationship hints must remain tenant-scoped and evidence-backed.

## F) Acceptance Tests
- AT-KG-01: Selene OS can invoke `KG_ENTITY_LINK` and output is schema-valid.
- AT-KG-02: Fact candidate ordering is bounded and deterministic for identical inputs.
- AT-KG-03: Entity/fact budget overflow fails closed deterministically.
- AT-KG-04: Fact-bundle validation drift fails closed before CONTEXT/NLP handoff.
