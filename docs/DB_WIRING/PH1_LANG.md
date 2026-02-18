# PH1_LANG DB Wiring (Design vNext)

## A) Engine Header
- engine_id: PH1.LANG
- layer: Understanding Assist
- authority: Non-Authoritative
- role: Multilingual detection/segmentation and response-language mapping across voice/text turns
- placement: TURN_OPTIONAL

## B) Ownership
- Tables owned: NONE (vNext design)
- Reads:
  - Structured, OS-supplied transcript input (`transcript_ok` or typed wrapper)
  - Optional locale/language preferences from session/profile context
- Writes: NONE (no direct persistence in vNext)

## C) Hard Boundaries
- Non-authoritative and non-executing; advisory output only.
- Must never grant authority, alter permissions, or bypass Access + Simulation ordering.
- Must never perform side effects, tool execution, or engine-to-engine direct calls.
- Hard no-translation rule: PH1.LANG must only detect/map language boundaries; it must never translate or rewrite user content.

## D) Wiring
- Invoked_by: OS step: pre-intent normalization pipeline for both voice and text paths
- Inputs_from: PH1.C transcript_ok (voice) or text transcript wrapper, optional locale hint and language preferences
- Outputs_to: language segmentation map + response-language plan returned to Selene OS and forwarded to PH1.SRL/PH1.NLP/PH1.X
- Invocation_condition: OPTIONAL(pre-intent multilingual pass; re-check on ambiguity)
- Deterministic sequence:
  - LANG_MULTIPLE_DETECT (detect language tags and bounded segment spans)
  - LANG_SEGMENT_RESPONSE_MAP (build response-language plan and self-validate consistency)
  - If validation_status != OK, OS refuses/fails closed and does not forward bundle.
- Not allowed:
  - Engine-to-engine direct calls.
  - Any execution commit or authority mutation.
  - Any bypass of Selene OS orchestration.

## E) Acceptance Tests
- AT-LANG-01: Selene OS can invoke capability_id and output is schema-valid.
- AT-LANG-02: Mixed-language segment boundaries are preserved with deterministic ordering.
- AT-LANG-03: Response-language map follows mode discipline (VOICE turn-level, TEXT segment-level).
- AT-LANG-04: Validation drift/budget overflow fails closed before PH1.SRL/PH1.NLP handoff.

## F) Related Engine Boundary (`PH1.SRL`)
- PH1.LANG output is advisory multilingual segmentation input to PH1.SRL only through Selene OS wiring.
- PH1.SRL remains deterministic owner of repaired transcript/frame output and may fail closed independent of PH1.LANG hints.
