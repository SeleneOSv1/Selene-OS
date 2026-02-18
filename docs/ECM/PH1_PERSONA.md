# PH1_PERSONA ECM (Design vNext)

## Engine Header
- engine_id: PH1.PERSONA
- role: Per-user personalization profile build/validate (tone/delivery hints only)
- placement: TURN_OPTIONAL

## Capability List

### capability_id: PERSONA_PROFILE_BUILD
- input_schema:
  - bounded envelope (`correlation_id`, `turn_id`, `max_signals`, `max_diagnostics`)
  - verified identity refs (`verified_user_id`, `verified_speaker_id`)
  - bounded preference signals with mandatory `evidence_ref`
  - bounded `correction_event_count`
  - optional `emo_guide_style_profile_ref`
  - optional `previous_snapshot_ref`
- output_schema:
  - `profile_snapshot`:
    - `style_profile_ref`
    - `delivery_policy_ref` (`VOICE_ALLOWED | TEXT_ONLY | SILENT`)
    - `brevity_ref` (`BRIEF | BALANCED | DETAILED`)
    - `preferences_snapshot_ref`
  - guard flags:
    - `auditable=true`
    - `tone_only=true`
    - `no_meaning_drift=true`
    - `no_execution_authority=true`
- allowed_callers: SELENE_OS_ONLY
- side_effects: NONE
- failure_modes: INPUT_SCHEMA_INVALID, IDENTITY_REQUIRED, BUDGET_EXCEEDED, INTERNAL_PIPELINE_ERROR
- reason_codes:
  - PH1_PERSONA_OK_PROFILE_BUILD
  - PH1_PERSONA_INPUT_SCHEMA_INVALID
  - PH1_PERSONA_IDENTITY_REQUIRED
  - PH1_PERSONA_BUDGET_EXCEEDED
  - PH1_PERSONA_INTERNAL_PIPELINE_ERROR

### capability_id: PERSONA_PROFILE_VALIDATE
- input_schema:
  - same bounded envelope + identity/signal inputs as PROFILE_BUILD
  - `proposed_profile_snapshot` from PROFILE_BUILD output
- output_schema:
  - `validation_status (OK|FAIL)`
  - bounded diagnostics
  - guard flags:
    - `auditable=true`
    - `tone_only=true`
    - `no_meaning_drift=true`
    - `no_execution_authority=true`
- allowed_callers: SELENE_OS_ONLY
- side_effects: NONE
- failure_modes: INPUT_SCHEMA_INVALID, IDENTITY_REQUIRED, BUDGET_EXCEEDED, VALIDATION_FAILED, INTERNAL_PIPELINE_ERROR
- reason_codes:
  - PH1_PERSONA_OK_PROFILE_VALIDATE
  - PH1_PERSONA_VALIDATION_FAILED
  - PH1_PERSONA_INPUT_SCHEMA_INVALID
  - PH1_PERSONA_IDENTITY_REQUIRED
  - PH1_PERSONA_BUDGET_EXCEEDED
  - PH1_PERSONA_INTERNAL_PIPELINE_ERROR

## Constraints
- Engines never call engines directly; Selene OS orchestrates all sequencing.
- PH1.PERSONA is non-authoritative; outputs are advisory only.
- Unknown identity must not be coerced into fallback persona output.
- Persona output cannot alter intent truth, permission decisions, confirmation flow, or execution order.

## Related Engine Boundaries
- PH1.EMO.GUIDE output may be consumed as optional style seed only.
- PH1.X consumption is limited to tone/phrasing posture.
- PH1.TTS consumption is limited to rendering policy selection.
- PH1.CACHE may consume `persona_profile_ref` as advisory ranking metadata only.
- PH1.LEARN/PH1.FEEDBACK may consume persona deltas only as non-authoritative learning signals.

## Sources
- `crates/selene_kernel_contracts/src/ph1persona.rs`
- `crates/selene_engines/src/ph1persona.rs`
- `crates/selene_os/src/ph1persona.rs`
