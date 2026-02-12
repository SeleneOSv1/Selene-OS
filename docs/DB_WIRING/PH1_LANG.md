# PH1_LANG DB Wiring (Design vNext)

## A) Engine Header
- engine_id: PH1.LANG
- layer: Understanding Assist
- authority: Non-Authoritative
- role: Multilingual detection/segmentation and language response mapping across STT and parse
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


## D) Wiring
- Invoked_by: OS step: pre-intent normalization pipeline for both voice and text paths
- Inputs_from: PH1.C transcript candidates (voice) or text transcript_ok-equivalent (text), plus optional PH1.SRL cues
- Outputs_to: language segmentation map + response language mapping returned to Selene OS and forwarded to PH1.SRL/PH1.NLP/PH1.X
- Invocation_condition: ALWAYS for parse path; optional re-check on ambiguity
- Not allowed:
  - Engine-to-engine direct calls.
  - Any execution commit or authority mutation.
  - Any bypass of Selene OS orchestration.

## E) Acceptance Tests
- AT-LANG-01: Mixed-language utterances in one turn are detected with correct segment boundaries.
- AT-LANG-02: Response language mapping matches user segments and selected response mode.
- AT-LANG-03: Broken/fragmented multilingual input is normalized before NLP intent parse.
