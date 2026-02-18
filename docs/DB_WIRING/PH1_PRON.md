# PH1_PRON DB Wiring (Design vNext)

## A) Engine Header
- engine_id: PH1.PRON
- layer: Speech Assist
- authority: Non-Authoritative
- role: Pronunciation enrollment and lexicon-pack building for names/acronyms/project terms
- placement: TURN_OPTIONAL

## B) Ownership
- Tables owned: NONE (vNext design)
- Reads:
  - Structured, OS-supplied pronunciation entries and scope metadata
  - Optional user/tenant context for scoping and consent checks
- Writes: NONE (no direct persistence in vNext)

## C) Hard Boundaries
- Non-authoritative and non-executing; advisory output only.
- Must never grant authority, alter permissions, or bypass Access + Simulation ordering.
- Must never perform side effects, tool execution, or engine-to-engine direct calls.
- Hard scope rule: all output is tenant-scoped; user-scoped packs require explicit consent.
- Hard meaning rule: pronunciation output may change speech rendering only and must never change semantic meaning.

## D) Wiring
- Invoked_by: OS step: pronunciation-assist phase for speech rendering and robustness hints
- Inputs_from: PH1.NLP/PH1.CONTEXT terms, explicit user corrections, bounded onboarding/enrollment cues
- Outputs_to:
  - pronunciation pack bundle for PH1.TTS
  - optional robustness hints for PH1.VOICE.ID and PH1.W
- Invocation_condition: OPTIONAL(pronunciation assist enabled)
- Deterministic sequence:
  - PRON_LEXICON_PACK_BUILD (build bounded tenant/user-scoped lexicon pack)
  - PRON_APPLY_VALIDATE (validate target-engine apply plan and fail closed on drift)
  - If validation_status != OK, OS refuses/fails closed and does not forward bundle.
- Not allowed:
  - Engine-to-engine direct calls.
  - Any execution commit or authority mutation.
  - Any bypass of Selene OS orchestration.

## E) Acceptance Tests
- AT-PRON-01: Selene OS can invoke capability_id and output is schema-valid.
- AT-PRON-02: User-scoped pronunciation packs are blocked without explicit consent.
- AT-PRON-03: Apply validation fails closed on locale/pack drift before PH1.TTS handoff.
- AT-PRON-04: Output remains bounded and deterministic for PH1.TTS/PH1.VOICE.ID/PH1.W consumers.

## F) Related Engine Boundary (`PH1.KNOW`)
- PH1.KNOW remains tenant dictionary/composition owner for enterprise vocabulary + pronunciation-hint bundles.
- PH1.PRON remains user/tenant pronunciation enrollment + lexicon owner for speech runtime robustness.
- Selene OS may merge PH1.KNOW and PH1.PRON outputs only after each engine's own validation path is `OK`; neither engine overrides the other's authority boundary.
