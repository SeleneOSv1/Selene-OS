# PH1_EMO_CORE DB Wiring (Design vNext)

## A) Engine Header
- engine_id: PH1.EMO.CORE
- implementation_id: PH1.EMO.CORE.001
- layer: Learning Assist
- authority: Non-Authoritative (tone/continuity only)
- role: Emotional snapshot/profile core with deterministic classify/reevaluate/privacy/tone/audit contracts
- placement: TURN_OPTIONAL

## B) Ownership
- Tables owned: NONE in this runtime slice.
- Reads:
  - OS-supplied verified identity and consent flags.
  - Bounded emotional signal bundle (`assertive_score`, `distress_score`, `anger_score`, `warmth_signal`).
  - Optional profile snapshot refs and reevaluation signal-window refs.
- Writes:
  - NONE directly in PH1.EMO.CORE runtime wiring.
  - Audit persistence remains owned by PH1.J/PH1.F; this module emits deterministic audit-event result packets only.

## C) Hard Boundaries
- PH1.EMO.CORE is advisory only; it must never grant authority, execute tools, or trigger side effects directly.
- Tone-only boundary is mandatory: outputs may shape delivery style/pacing only; meaning must not drift.
- Consent and identity are strict gates for profile classification/reevaluation and privacy command handling.
- Destructive privacy commands require explicit confirmation.
- Snapshot capture is non-blocking: when consent/identity is missing, return deterministic `DEFER` (fail-safe continuity behavior).
- Engines never call engines directly; Selene OS orchestrates all sequencing.

## D) Wiring
- Invoked_by: Selene OS in adaptation/onboarding windows.
- Inputs_from:
  - Identity context (`identity_verified`) and consent policy flags (`consent_asserted`).
  - Optional onboarding session context for snapshot capture.
  - Optional profile snapshot and signal-window references.
- Outputs_to:
  - Selene OS tone planning handoff to PH1.X/PH1.TTS (advisory only, after validation).
  - PH1.PERSONA as bounded optional context (reference-only).
- Deterministic capability set:
  - `PH1EMO_CLASSIFY_PROFILE_COMMIT_ROW` (`EMO_SIM_001`)
  - `PH1EMO_REEVALUATE_PROFILE_COMMIT_ROW` (`EMO_SIM_002`)
  - `PH1EMO_PRIVACY_COMMAND_COMMIT_ROW` (`EMO_SIM_003`)
  - `PH1EMO_TONE_GUIDANCE_DRAFT_ROW` (`EMO_SIM_004`)
  - `PH1EMO_SNAPSHOT_CAPTURE_COMMIT_ROW` (`EMO_SIM_005`)
  - `PH1EMO_AUDIT_EVENT_COMMIT_ROW` (`EMO_SIM_006`)

## E) Related Engine Boundaries
- PH1.EMO.CORE is a concrete emotional implementation surface and is invoked directly by Selene OS/blueprints.
- PH1.EMO.GUIDE remains a separate concrete tone-policy assist surface for pre-response style hints.
- PH1.X and PH1.TTS may consume PH1.EMO.CORE outputs only as tone-policy hints (`tone_only=true`, `no_meaning_drift=true`).
- PH1.PERSONA may consume PH1.EMO.CORE snapshot refs as advisory context only.

## F) Acceptance Tests
- AT-EMO-CORE-01: classify profile commit requires consent + verified identity.
- AT-EMO-CORE-02: reevaluate profile commit is deterministic for identical signal-window refs.
- AT-EMO-CORE-03: destructive privacy command requires explicit confirmation.
- AT-EMO-CORE-04: tone guidance draft emits tone-only advisory bundle with no meaning drift.
- AT-EMO-CORE-05: snapshot capture returns `DEFER` (not fail) when consent/identity is missing.
- AT-EMO-CORE-06: snapshot capture returns deterministic `snapshot_ref` on complete path.
- AT-EMO-CORE-07: audit event commit emits deterministic `event_id` for identical inputs.
- AT-EMO-CORE-08: simulation/capability drift is refused by OS wiring.

## G) Implementation References
- Kernel contracts: `crates/selene_kernel_contracts/src/ph1emocore.rs`
- Engine runtime: `crates/selene_engines/src/ph1emocore.rs`
- OS wiring: `crates/selene_os/src/ph1emocore.rs`
