# PH1_EMO_GUIDE DB Wiring (Design vNext)

## A) Engine Header
- engine_id: PH1.EMO.GUIDE
- layer: Learning Assist
- authority: Non-Authoritative (tone policy only)
- role: Deterministic style profile guidance (`DOMINANT | GENTLE` + bounded modifiers)
- placement: TURN_OPTIONAL

## B) Ownership
- Tables owned: NONE (runtime guidance slice)
- Reads:
  - Structured, OS-supplied verified `speaker_id` and bounded interaction signals (`interaction_count`, `correction_events`, `interruption_events`, `assertive_events`, `cooperative_events`).
  - Optional bounded `emo_core_snapshot_ref` (read-only reference) for alignment with PH1.EMO.CORE.
- Writes: NONE (no direct persistence in PH1.EMO.GUIDE runtime wiring)

## B1) Phone-First Artifact Custody (Required Extension)
- `PH1.EMO.GUIDE` must execute phone-first when emotional guidance is enabled.
- Guidance artifacts/policy pointers must be present locally on phone (`ACTIVE + N-1 rollback`) and continuously synced to Selene.
- Engine B owns outbox/vault replay/ack semantics; PH1.EMO.GUIDE owns deterministic guidance artifact-manifest delta emission contract.

## C) Hard Boundaries
- Tone policy only: output can influence style selection, pacing hints, and voice rendering profile hints only.
- Must never alter facts, intent, permissions, confirmation requirements, or execution order.
- Must never execute tools, simulations, or side effects.
- Engines never call engines directly; Selene OS orchestrates all sequencing.
- If profile-build/profile-validate integrity drifts, PH1.EMO.GUIDE must fail closed before downstream handoff.

## D) Wiring
- Invoked_by: Selene OS optional adaptation window (pre-response planning).
- Inputs_from:
  - Verified identity context (speaker binding already resolved upstream).
  - Interaction/correction/interrupt summaries supplied by Selene OS.
  - Optional PH1.EMO.CORE snapshot reference.
- Outputs_to:
  - `emo_guide_profile_bundle` returned to Selene OS.
  - Selene OS may forward bounded style hints to PH1.X and PH1.TTS only after validation passes.
  - Engine B outbox handoff for guidance artifact-manifest deltas (required extension; no direct PH1.EMO.GUIDE storage writes).
- Invocation_condition: TURN_OPTIONAL (emo-guide assist enabled)
- Deterministic sequence:
  - `EMO_GUIDE_PROFILE_BUILD`:
    - Computes one style profile (`DOMINANT` or `GENTLE`).
    - Emits canonical modifier set (`BRIEF | WARM | FORMAL`, ordered, deduped, bounded).
    - Emits guard flags: `tone_only=true`, `no_meaning_drift=true`, `no_execution_authority=true`, `auditable=true`, `reversible=true`.
  - `EMO_GUIDE_PROFILE_VALIDATE`:
    - Recomputes expected profile from identical bounded inputs.
    - Verifies profile/modifier/stability determinism and emits `validation_status`.
  - If `validation_status != OK`, Selene OS fails closed and must not forward PH1.EMO.GUIDE output.

## E) Related Engine Boundaries
- PH1.X may consume PH1.EMO.GUIDE style hints only for response tone shaping; conversation meaning must remain unchanged.
- PH1.TTS may consume style profile/modifier hints only as rendering policy (`style_profile_ref` + modifiers).
- PH1.EMO.CORE remains a separate module; PH1.EMO.GUIDE may read bounded references but cannot override PH1.EMO.CORE truth.
- PH1.PERSONA may persist selected style profile snapshots; PH1.EMO.GUIDE itself remains no-write in this slice.

## F) Acceptance Tests
- AT-EMO-GUIDE-01: Selene OS can invoke `EMO_GUIDE_PROFILE_BUILD` and output is schema-valid.
- AT-EMO-GUIDE-02: Identical inputs produce identical `style_profile_ref` + modifier order.
- AT-EMO-GUIDE-03: Profile validation drift fails closed before PH1.X/PH1.TTS handoff.
- AT-EMO-GUIDE-04: Guard flags (`tone_only`, `no_meaning_drift`, `no_execution_authority`) are always true on forwarded outputs.
- AT-EMO-GUIDE-05: phone-local guidance artifact pointer and cloud sync cursor reconcile deterministically.
- AT-EMO-GUIDE-06: guidance artifact sync enqueue is idempotent and ack-gated.
