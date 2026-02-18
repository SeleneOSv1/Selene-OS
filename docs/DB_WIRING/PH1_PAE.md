# PH1_PAE DB Wiring (Design vNext)

## A) Engine Header
- engine_id: PH1.PAE
- layer: Learning Assist
- authority: Non-Authoritative
- role: Provider arbitration score build + adaptation hint emission (`SHADOW|ASSIST|LEAD`)
- placement: TURN_OPTIONAL (async adaptation window)

## B) Ownership
- Tables owned: NONE (vNext runtime design)
- Reads:
  - Structured, OS-supplied LISTEN/FEEDBACK/LEARN signals.
  - Governance-approved RLL-derived artifact signals only.
  - Bounded mode/policy budget context (`current_mode`, sample/threshold settings).
- Writes: NONE (no direct persistence in vNext runtime wiring)

## C) Hard Boundaries
- Non-authoritative and non-executing; outputs are advisory routing/adaptation hints only.
- Must never grant authority, alter permissions, or bypass Access + Simulation ordering.
- Must never perform side effects, tool execution, or engine-to-engine direct calls.
- Hard rule: promotion to `LEAD` requires governed artifact + rollback pointer discipline.
- Hard rule: repeated regression must demote deterministically (`LEAD -> ASSIST -> SHADOW`) instead of drifting silently.
- Hard rule: if selected/ordered score integrity drifts, PH1.PAE must fail closed before downstream handoff.

## D) Wiring
- Invoked_by: Selene OS optional adaptation window (post-turn and/or pre-next-turn planning window).
- Inputs_from:
  - PH1.LISTEN validated adaptation context (`LISTEN_SIGNAL_FILTER=OK`).
  - PH1.FEEDBACK validated correction/confidence signals (`FEEDBACK_SIGNAL_EMIT=OK`).
  - PH1.LEARN validated package signals (`LEARN_ARTIFACT_PACKAGE_BUILD=OK`).
  - PH1.RLL-derived signals only when governance has activated the artifact.
- Outputs_to:
  - `pae_policy_bundle` returned to Selene OS.
  - Selene OS may forward bounded route/adaptation hints to PH1.C, PH1.TTS, PH1.CACHE, and PH1.MULTI.
- Invocation_condition: TURN_OPTIONAL (adaptation policy enabled)
- Deterministic sequence:
  - `PAE_POLICY_SCORE_BUILD`:
    - Scores bounded candidate plans using deterministic weighted signal inputs.
    - Produces one selected candidate + ordered score list + selected mode.
    - Enforces governed-artifact requirements and promotion/demotion constraints.
  - `PAE_ADAPTATION_HINT_EMIT`:
    - Validates selected-vs-ordered score integrity.
    - Emits bounded target-engine hints only (no execution directive).
    - Returns `validation_status (OK|FAIL)` and bounded diagnostics.
  - If `validation_status != OK`, Selene OS fails closed and must not forward PAE output.
- Not allowed:
  - Engine-to-engine direct calls.
  - Any execution commit or authority mutation.
  - Any bypass of Selene OS orchestration.

## E) Related Engine Boundaries
- PH1.FEEDBACK may influence PH1.PAE scoring only through validated advisory signal output.
- PH1.LEARN may influence PH1.PAE scoring only through validated governed package outputs.
- PH1.LISTEN may influence PH1.PAE scoring only as advisory environment context.
- PH1.RLL remains OFFLINE_ONLY; PH1.PAE may consume RLL-derived signals only after governance activation.
- PH1.CACHE consumes PAE output only as bounded route/cost bias metadata; cache gating requirements remain canonical in PH1.CACHE.

## F) Acceptance Tests
- AT-PAE-01: Selene OS can invoke `PAE_POLICY_SCORE_BUILD` and output is schema-valid.
- AT-PAE-02: Candidate score ordering is bounded and deterministic for identical inputs.
- AT-PAE-03: Promotion to `LEAD` requires governed artifact + rollback discipline; otherwise fail closed/demote.
- AT-PAE-04: Hint-emit validation drift fails closed before PH1.C/PH1.TTS/PH1.CACHE/PH1.MULTI handoff.
