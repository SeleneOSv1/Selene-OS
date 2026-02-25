# PH1_FEEDBACK DB Wiring (Design vNext)

## A) Engine Header
- engine_id: PH1.FEEDBACK
- layer: Learning Assist
- authority: Non-Authoritative
- role: Structured correction/confidence feedback capture and signal emission
- placement: TURN_OPTIONAL

## B) Ownership
- Tables owned: NONE (vNext design)
- Reads:
  - Structured, OS-supplied post-turn outcomes (`transcript_reject`, correction events, clarify loops, tool failures, interruption events).
  - Bounded evidence refs only.
- Writes: NONE (no direct persistence in vNext runtime wiring)

## C) Hard Boundaries
- Non-authoritative and non-executing; output is advisory signals only.
- Must never grant authority, alter permissions, or bypass Access + Simulation ordering.
- Must never perform side effects, tool execution, or engine-to-engine direct calls.
- Hard rule: feedback capture may produce learning signals only; it must not change runtime execution outcomes directly.

## D) Wiring
- Invoked_by: Selene OS post-turn feedback window or explicit correction flow.
- Inputs_from:
  - PH1.C reject/retry outcomes.
  - PH1.NLP/PH1.X clarify/confirm abort outcomes.
  - PH1.E tool failure/conflict outcomes.
  - PH1.X/PH1.TTS barge-in and delivery-switch outcomes.
- Outputs_to: `feedback_signal_bundle` returned to Selene OS and forwarded to PH1.LEARN package builders and PH1.PAE `PAE_POLICY_SCORE_BUILD` score inputs.
- Invocation_condition: TURN_OPTIONAL (post-turn async window)
- Deterministic sequence:
  - `FEEDBACK_EVENT_COLLECT` builds selected + ordered signal candidates.
  - `FEEDBACK_SIGNAL_EMIT` self-validates rank/selection/emit-target integrity.
  - If `validation_status != OK`, Selene OS fails closed and does not forward feedback bundle.
- Not allowed:
  - Engine-to-engine direct calls.
  - Any execution commit or authority mutation.
  - Any bypass of Selene OS orchestration.

## E) Related Engine Boundaries
- PH1.LEARN consumes FEEDBACK signals as package inputs only; PH1.FEEDBACK does not own artifact activation.
- PH1.LEARN runtime contract surface is defined in `docs/DB_WIRING/PH1_LEARN.md` and `docs/ECM/PH1_LEARN.md`.
- PH1.PAE may consume FEEDBACK-derived score inputs only as advisory policy hints via `PAE_POLICY_SCORE_BUILD`.
- Existing append-only storage contracts for feedback/artifacts remain canonical in `docs/DB_WIRING/PH1_LEARN_FEEDBACK_KNOW.md`.

## F) Acceptance Tests
- AT-FEEDBACK-01: Selene OS can invoke `FEEDBACK_EVENT_COLLECT` and output is schema-valid.
- AT-FEEDBACK-02: Signal candidate ordering is bounded and deterministic.
- AT-FEEDBACK-03: Event/signal budget overflow fails closed deterministically.
- AT-FEEDBACK-04: Signal-emit validation drift fails closed before LEARN/PAE handoff.

## G) FDX Wiring Lock (Section 5F)
- PH1.FEEDBACK wiring must capture duplex incident signals with deterministic ordering and bounded evidence refs.
- Required FDX signal families: false interrupt, missed interrupt, late cancel, low-confidence transcript fallback, continuity clarify fallback.
- All FDX signals remain advisory-only and flow through Selene OS into PH1.LEARN/PH1.PAE.
