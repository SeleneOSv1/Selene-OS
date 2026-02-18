# PH1_KNOW DB Wiring (Design vNext)

## A) Engine Header
- engine_id: PH1.KNOW
- layer: Knowledge Assist
- authority: Non-Authoritative
- role: Tenant dictionary and pronunciation-hint pack composition
- placement: TURN_OPTIONAL

## B) Ownership
- Tables owned: NONE (vNext runtime design)
- Reads:
  - Structured, OS-supplied tenant terms from authorized HR/org sources.
  - Structured, OS-supplied user-provided terms only when explicit consent is asserted.
  - Optional LEARN artifact references supplied by Selene OS.
- Writes: NONE (no direct persistence in vNext runtime wiring)
- Persistence boundary:
  - PH1.KNOW artifact ledger writes are canonical in `docs/DB_WIRING/PH1_LEARN_FEEDBACK_KNOW.md` (`created_by=PH1.KNOW`).

## C) Hard Boundaries
- Non-authoritative and non-executing; output is advisory hints only.
- Must never grant authority, alter permissions, or bypass Access + Simulation ordering.
- Must never perform side effects, tool execution, or engine-to-engine direct calls.
- Hard rule: tenant scope is mandatory; cross-tenant entries must fail closed.
- Hard rule: authorized sources only; unverified terms must fail closed.
- Hard rule: user-provided terms require explicit consent assertion.
- Hard rule: PH1.KNOW may influence vocabulary/pronunciation only and must never mutate semantic meaning.

## D) Wiring
- Invoked_by: Selene OS when dictionary/terminology assist is enabled.
- Inputs_from:
  - Authorized HR/org term set.
  - Explicitly consented user-provided terms.
  - Optional LEARN artifact references.
- Outputs_to:
  - bounded vocabulary hints for PH1.C, PH1.SRL, PH1.NLP.
  - pronunciation-hint subset for PH1.TTS (only when available).
- Invocation_condition: OPTIONAL (knowledge assist enabled)
- Deterministic sequence:
  - `KNOW_DICTIONARY_PACK_BUILD`
    - dedupes and canonical-orders tenant-scoped terms.
    - enforces authorization + consent + tenant scope.
  - `KNOW_HINT_BUNDLE_SELECT`
    - validates target-engine handoff and canonical ordering.
    - validates tenant-scope/authorized-only/no-cross-tenant preservation.
  - If `validation_status != OK`, Selene OS fails closed and does not forward KNOW output.
- Not allowed:
  - Engine-to-engine direct calls.
  - Any execution commit or authority mutation.
  - Any bypass of Selene OS orchestration.

## E) Related Engine Boundaries
- PH1.C / PH1.SRL / PH1.NLP may consume PH1.KNOW hints only through Selene OS and only when `KNOW_HINT_BUNDLE_SELECT=OK`.
- PH1.TTS may consume pronunciation hints from PH1.KNOW only when present in validated output; no semantic rewrite is allowed.
- PH1.PRON remains pronunciation enrollment + lexicon-pack owner; PH1.KNOW remains tenant dictionary/composition owner.
- PH1.LEARN may feed artifact references into PH1.KNOW input; PH1.KNOW runtime output stays advisory.
- PH1.KG may consume PH1.KNOW artifacts only as tenant-scoped seed metadata through Selene OS.

## F) Acceptance Tests
- AT-KNOW-01: Selene OS can invoke capability_id and output is schema-valid.
- AT-KNOW-02: Unverified/unauthorized source entries fail closed deterministically.
- AT-KNOW-03: Cross-tenant entry leakage is blocked deterministically.
- AT-KNOW-04: TTS target validation fails closed when pronunciation hints are missing.
