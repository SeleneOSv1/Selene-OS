# PH1_LISTEN DB Wiring (Design vNext)

## A) Engine Header
- engine_id: PH1.LISTEN
- layer: Learning Assist
- authority: Non-Authoritative
- role: Active listening + environment adaptation hints
- placement: TURN_OPTIONAL

## B) Ownership
- Tables owned: NONE (vNext runtime design)
- Reads:
  - Structured, OS-supplied PH1.K perception windows (VAD confidence, speech-likeness, noise level, overlap/silence timing).
  - Structured, OS-supplied correction snapshot inputs (user corrections, delivery switches, barge-in counts).
  - Structured, OS-supplied session context flags (meeting/car/privacy/text preference).
- Writes: NONE (no direct persistence in vNext runtime wiring)

## C) Hard Boundaries
- Non-authoritative and non-executing; output is advisory adjustment metadata only.
- Must never grant authority, alter permissions, or bypass Access + Simulation ordering.
- Must never perform side effects, tool execution, or engine-to-engine direct calls.
- Hard rule: LISTEN may affect capture and delivery mode hints only; it must never change intent, transcript meaning, or execution decisions.
- Hard rule: if selected profile/order integrity drifts, LISTEN must fail closed before any downstream handoff.

## D) Wiring
- Invoked_by: Selene OS optional adaptation window (post-turn and/or pre-next-turn planning window).
- Inputs_from:
  - PH1.K bounded perception windows.
  - PH1.FEEDBACK bounded correction summary fields.
  - Session/policy context supplied by Selene OS.
- Outputs_to:
  - `listen_adjustment_bundle` returned to Selene OS.
  - Selene OS may forward bounded capture/endpoint hints to PH1.ENDPOINT/PH1.C.
  - Selene OS may forward bounded `environment_profile_ref`/delivery hints to PH1.PAE and PH1.MULTI.
- Invocation_condition: TURN_OPTIONAL (listening adaptation enabled)
- Deterministic sequence:
  - `LISTEN_SIGNAL_COLLECT`:
    - Classifies one environment mode (`quiet|noisy|meeting|car|office`).
    - Produces `environment_profile_ref` + selected adjustment + ordered adjustment hints.
  - `LISTEN_SIGNAL_FILTER`:
    - Validates selected-vs-ordered integrity, profile consistency, and no-meaning-mutation boundary.
  - If `validation_status != OK`, Selene OS fails closed and must not forward LISTEN output.
- Not allowed:
  - Engine-to-engine direct calls.
  - Any execution commit or authority mutation.
  - Any bypass of Selene OS orchestration.

## E) Related Engine Boundaries
- PH1.K remains the runtime perception source; PH1.LISTEN does not own capture runtime state.
- PH1.ENDPOINT may consume LISTEN hints only as bounded boundary/capture tuning input after `LISTEN_SIGNAL_FILTER=OK`.
- PH1.PAE consumes LISTEN output as advisory adaptation input only through `PAE_POLICY_SCORE_BUILD`; no authority/execute path is introduced.
- PH1.MULTI may include LISTEN environment hints as context metadata only; meaning mutation remains forbidden.

## F) Acceptance Tests
- AT-LISTEN-01: Selene OS can invoke `LISTEN_SIGNAL_COLLECT` and output is schema-valid.
- AT-LISTEN-02: Environment/adaptation ordering is bounded and deterministic for identical inputs.
- AT-LISTEN-03: Signal/adjustment budget overflow fails closed deterministically.
- AT-LISTEN-04: Filter validation drift fails closed before ENDPOINT/PAE/MULTI handoff.
