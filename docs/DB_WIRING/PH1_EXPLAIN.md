# PH1_EXPLAIN DB Wiring (Design vNext)

## A) Engine Header
- engine_id: PH1.EXPLAIN
- layer: Conversation Assist
- authority: Non-Authoritative
- role: Deterministic reason-coded explanation packet generation for explicit why/how/what-happened requests
- placement: TURN_OPTIONAL (explicit explain trigger only)

## B) Ownership
- Tables owned: NONE in this runtime slice.
- Reads:
  - bounded explain request (`request_type`, optional short utterance)
  - bounded event context (`primary_reason_code`, optional directive context, optional `verbatim_trigger`)
  - optional memory evidence candidate (`evidence_quote`, provenance, sensitivity flag)
  - policy context (`privacy_mode`, `do_not_disturb`, safety tier)
- Writes:
  - no direct table writes in this runtime slice
  - emits deterministic explanation or explanation_refuse payload only

## C) Hard Boundaries
- PH1.EXPLAIN is advisory only; it never executes actions, tools, or simulations.
- PH1.EXPLAIN never grants authority and never mutates gate order.
- PH1.EXPLAIN must never leak providers, thresholds, raw scores, prompt/system internals, or chain-of-thought.
- PH1.EXPLAIN must never invent causes; explanations must be reason-code and evidence grounded.
- Memory evidence quoting is privacy-gated; sensitive/private contexts must return `explanation_refuse`.
- Explanation text remains bounded to 1–2 sentences.

## D) Wiring
- Invoked_by: Selene OS when user explicitly asks why/how/what happened, or when accountability text is requested by policy.
- Inputs_from:
  - PH1.X context (`conversation_directive` for clarify/confirm context)
  - PH1.J reason-coded event outcomes
  - PH1.M candidate evidence (optional, policy-gated)
  - PH1.K / PH1.TTS interruption triggers (`verbatim_trigger` optional)
- Outputs_to:
  - explanation packet returned to Selene OS for optional PH1.X/PH1.WRITE surfacing
- Invocation_condition: explicit explain request only (`WHY`, `WHY_NOT`, `HOW_KNOW`, `WHAT_NEXT`, `WHAT_HAPPENED`)
- Deterministic sequence:
  - `EXPLAIN_REASON_RENDER`:
    - derives short explanation from directive context or reason-code class mapping.
    - enforces one/two-sentence bound and no-internal-leak wording.
  - `EXPLAIN_EVIDENCE_SELECT`:
    - if `HOW_KNOW` and memory candidate is present, performs privacy gate and either emits evidence-backed explanation or `explanation_refuse`.
- Not allowed:
  - engine-to-engine direct calls
  - side effects
  - authority mutation
  - disclosure of blocked sensitive evidence

## E) Related Engine Boundaries
- `PH1.X`: owns whether explanation output is surfaced to the user; PH1.EXPLAIN only produces packet candidates.
- `PH1.J`: provides reason-code context used for deterministic explanation mapping.
- `PH1.M`: optional evidence source; PH1.EXPLAIN enforces privacy-safe quoting.
- `PH1.C` / `PH1.W` / `PH1.D` / `PH1.E` / `PH1.L` / `PH1.K` / `PH1.TTS` / `PH1.VOICE.ID`: reason-code namespaces consumed for category-specific explanations.

## F) Acceptance Tests
- AT-EX-01: “Why did you ask?” explanation cites missing field (no guessing).
- AT-EX-02: “Why didn’t you proceed?” explanation cites confirmation gate when applicable.
- AT-EX-03: “How do you know?” returns evidence-backed explanation or privacy refusal.
- AT-EX-04: “Why did you stop?” cites interrupt phrase when `verbatim_trigger` is present.
- AT-EX-05: no internal leakage on STT/tool/LLM failures.
- AT-EX-06: output stays within one/two-sentence bound.
