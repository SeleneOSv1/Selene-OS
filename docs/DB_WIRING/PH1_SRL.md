# PH1_SRL DB Wiring (Design vNext)

## A) Engine Header
- engine_id: PH1.SRL
- layer: Understanding
- authority: Non-Authoritative
- role: Post-STT semantic repair layer that improves messy but trustworthy transcripts without changing meaning
- placement: ALWAYS_ON (after PH1.C transcript_ok)

## B) Ownership
- Tables owned: NONE (vNext runtime design)
- Reads:
  - Structured, OS-supplied `transcript_ok` payload from PH1.C.
  - Optional uncertain-span hints from PH1.C and optional tenant dictionary hints from PH1.KNOW via Selene OS.
  - Optional language hints from PH1.LANG via Selene OS.
- Writes: NONE (no direct persistence in vNext runtime wiring)

## C) Hard Boundaries
- Non-authoritative and non-executing; outputs are advisory normalized frames only.
- Must never grant authority, alter permissions, or bypass Access + Simulation ordering.
- Must never perform side effects, tool execution, or engine-to-engine direct calls.
- Hard rule: PH1.SRL may normalize punctuation/shorthand only when deterministic.
- Hard rule: PH1.SRL must preserve code-switch spans and scripts verbatim.
- Hard rule: PH1.SRL must never invent fields, translate without explicit request, or change intent.

## D) Wiring
- Invoked_by: Selene OS immediately after PH1.C emits `transcript_ok`.
- Inputs_from:
  - PH1.C transcript + uncertainty hints.
  - PH1.LANG multilingual segmentation hints (optional).
  - PH1.KNOW dictionary hints (optional, advisory only).
- Outputs_to:
  - `srl_frame` returned to Selene OS and forwarded to PH1.NLP.
- Invocation_condition: ALWAYS_ON
- Deterministic sequence:
  - `SRL_FRAME_BUILD`
    - token spans, deterministic shorthand normalization, role hints, ambiguity flags.
    - preserves code-switch and no-translation boundary.
  - `SRL_ARGUMENT_NORMALIZE`
    - validates ordering/overlap/normalization safety and returns `validation_status` + diagnostics.
    - requires clarify when ambiguity flags remain.
  - If `validation_status != OK`, Selene OS fails closed and does not forward SRL bundle.
- Not allowed:
  - Engine-to-engine direct calls.
  - Any execution commit or authority mutation.
  - Any bypass of Selene OS orchestration.

## E) Related Engine Boundaries
- PH1.LANG -> PH1.SRL: language segmentation hints are advisory; PH1.SRL owns deterministic repair output.
- PH1.KNOW -> PH1.SRL: dictionary hints are advisory only and must remain tenant-scoped/authorized.
- PH1.SRL -> PH1.NLP: PH1.NLP remains deterministic owner of final `intent_draft|clarify|chat` decision.
- PH1.SRL ambiguity notes are consumed by PH1.NLP deterministic clarify flow only; SRL frame truth remains immutable.

## F) Acceptance Tests
- AT-SRL-01: No new facts introduced.
- AT-SRL-02: Code-switch preserved.
- AT-SRL-03: Ambiguity -> clarify, not inference.
- AT-SRL-04: Budget/validation drift fails closed before NLP handoff.
