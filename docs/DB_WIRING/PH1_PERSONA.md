# PH1_PERSONA DB Wiring (Design vNext)

## A) Engine Header
- engine_id: PH1.PERSONA
- layer: Learning Assist
- authority: Non-Authoritative
- role: Per-user personalization profile build/validate (`style_profile_ref`, `delivery_policy_ref`, `preferences_snapshot_ref`)
- placement: TURN_OPTIONAL (identity-verified adaptation window)

## B) Ownership
- Tables owned: NONE (runtime wiring slice)
- Reads:
  - Structured, OS-supplied verified identity inputs (`verified_user_id`, `verified_speaker_id`).
  - Bounded preference signals (`preferred_language`, `brevity`, `response_tone_target`, `privacy`, `confirmation`) with mandatory `evidence_ref`.
  - Optional PH1.EMO.GUIDE style hint (`emo_guide_style_profile_ref`) and optional previous persona snapshot ref.
- Writes: NONE (no direct PH1.PERSONA table writes in runtime wiring)
- Persistence boundary:
  - Canonical append-only persona persistence remains in the locked storage row:
    - `docs/DB_WIRING/PH1_PERSONA.md` (this file) + `crates/selene_storage/src/ph1f.rs` + `crates/selene_storage/src/repo.rs` (`Ph1PersonaRepo`)
  - Runtime contracts do not bypass PH1.F/PH1.J append-only discipline.

## C) Hard Boundaries
- PH1.PERSONA is advisory only and must never grant authority or execute actions.
- Unknown identity must fail closed at OS wiring (`NotInvokedIdentityUnknown`), with no persona hints forwarded.
- Persona output may influence tone/delivery only; it must never alter factual meaning, intent decisions, confirmations, access outcomes, or simulation order.
- Every accepted preference signal must be evidence-backed; no silent preference inference is permitted.
- Engines never call engines directly; Selene OS orchestrates all sequencing.

## D) Wiring
- Invoked_by: Selene OS in post-identity pre-response adaptation window.
- Inputs_from:
  - Identity context after PH1.VOICE.ID or signed-in text identity binding.
  - PH1.EMO.GUIDE validated style hint (optional).
  - User preference signals/corrections and optional previous persona snapshot ref.
- Outputs_to:
  - `persona_profile_bundle` returned to Selene OS.
  - Selene OS may forward bounded hints to PH1.X and PH1.TTS only after validation passes.
  - Selene OS may forward `persona_profile_ref` to PH1.CACHE as advisory planning input.
- Invocation_condition: TURN_OPTIONAL (persona assist enabled + identity verified)
- Deterministic sequence:
  - `PERSONA_PROFILE_BUILD`
    - computes deterministic profile snapshot from bounded signals.
    - emits guard flags: `auditable=true`, `tone_only=true`, `no_meaning_drift=true`, `no_execution_authority=true`.
  - `PERSONA_PROFILE_VALIDATE`
    - recomputes expected snapshot from the same bounded inputs.
    - validates deterministic parity and returns `validation_status (OK|FAIL)` + bounded diagnostics.
  - if `validation_status != OK`, Selene OS fails closed and must not forward persona hints downstream.
- Not allowed:
  - engine-to-engine direct calls
  - direct execution authority mutation
  - bypass of Selene OS orchestration, Access gate, or Simulation gate ordering

## E) Related Engine Boundaries
- PH1.EMO.GUIDE -> PH1.PERSONA:
  - EMO style hints are optional seed input only.
  - explicit user preference signals may override EMO hints deterministically.
- PH1.PERSONA -> PH1.X:
  - PH1.X may consume persona hints only for phrasing/tone/delivery posture.
  - PH1.X must not allow persona hints to change truth, clarify logic, confirmation, or dispatch semantics.
- PH1.PERSONA -> PH1.TTS:
  - PH1.TTS may consume persona style/delivery hints only as render policy inputs.
  - PH1.TTS must preserve semantic meaning and execution safety regardless of persona hints.
- PH1.PERSONA -> PH1.CACHE:
  - `persona_profile_ref` is advisory-only ranking input and cannot bypass gate requirements.
- PH1.LEARN/PH1.FEEDBACK:
  - persona preference deltas may feed learning artifacts, but learning remains non-authoritative and activation-governed.

## F) Acceptance Tests
- AT-PERS-01: unknown speaker -> no persona applied (`NotInvokedIdentityUnknown`).
- AT-PERS-02: preference updates require evidence-backed signals and auditable outputs.
- AT-PERS-03: persona hints are tone/delivery only (`tone_only=true`, `no_meaning_drift=true`, `no_execution_authority=true`).
- AT-PERS-04: build/validate drift fails closed before PH1.X/PH1.TTS/PH1.CACHE forwarding.

## G) Implementation References
- Kernel contracts: `crates/selene_kernel_contracts/src/ph1persona.rs`
- Engine runtime: `crates/selene_engines/src/ph1persona.rs`
- OS wiring: `crates/selene_os/src/ph1persona.rs`
- Storage row lock: `crates/selene_storage/src/ph1f.rs`, `crates/selene_storage/src/repo.rs`, `crates/selene_storage/tests/ph1_persona/db_wiring.rs`
