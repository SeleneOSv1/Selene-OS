# PH1_EMO_GUIDE ECM (Design vNext)

## Engine Header
- engine_id: PH1.EMO.GUIDE
- role: Deterministic emotional guidance style classification and validation
- placement: TURN_OPTIONAL

## Capability List

### capability_id: EMO_GUIDE_PROFILE_BUILD
- input_schema:
  - bounded envelope (`correlation_id`, `turn_id`, `max_interactions`, `max_modifiers`, `max_diagnostics`)
  - `verified_speaker_id`
  - bounded interaction signals (`interaction_count`, `correction_events`, `interruption_events`, `assertive_events`, `cooperative_events`)
  - optional `emo_core_snapshot_ref` (bounded)
- output_schema:
  - `style_profile_ref` (`DOMINANT | GENTLE`)
  - ordered/deduped modifiers (`BRIEF | WARM | FORMAL`)
  - `stability_window_turns`
  - guard flags: `tone_only=true`, `no_meaning_drift=true`, `auditable=true`, `reversible=true`, `no_execution_authority=true`
- allowed_callers: SELENE_OS_ONLY
- side_effects: NONE
- failure_modes: INPUT_SCHEMA_INVALID, IDENTITY_REQUIRED, BUDGET_EXCEEDED, INTERNAL_PIPELINE_ERROR
- reason_codes:
  - PH1_EMO_GUIDE_OK_PROFILE_BUILD
  - PH1_EMO_GUIDE_INPUT_SCHEMA_INVALID
  - PH1_EMO_GUIDE_IDENTITY_REQUIRED
  - PH1_EMO_GUIDE_BUDGET_EXCEEDED
  - PH1_EMO_GUIDE_INTERNAL_PIPELINE_ERROR

### capability_id: EMO_GUIDE_PROFILE_VALIDATE
- input_schema:
  - same bounded envelope + speaker/signal inputs as profile build
  - optional `emo_core_snapshot_ref`
  - `proposed_profile` from `EMO_GUIDE_PROFILE_BUILD`
- output_schema:
  - `validation_status` (`OK | FAIL`)
  - bounded diagnostics
  - guard flags: `tone_only=true`, `no_meaning_drift=true`, `no_execution_authority=true`
- allowed_callers: SELENE_OS_ONLY
- side_effects: NONE
- failure_modes: INPUT_SCHEMA_INVALID, IDENTITY_REQUIRED, BUDGET_EXCEEDED, VALIDATION_FAILED, INTERNAL_PIPELINE_ERROR
- reason_codes:
  - PH1_EMO_GUIDE_OK_PROFILE_VALIDATE
  - PH1_EMO_GUIDE_VALIDATION_FAILED
  - PH1_EMO_GUIDE_INPUT_SCHEMA_INVALID
  - PH1_EMO_GUIDE_IDENTITY_REQUIRED
  - PH1_EMO_GUIDE_BUDGET_EXCEEDED
  - PH1_EMO_GUIDE_INTERNAL_PIPELINE_ERROR

## Constraints
- Non-authoritative assist engine; outputs are advisory only.
- Hard rule: tone policy only; PH1.EMO.GUIDE must not modify intent truth or execution semantics.
- Hard rule: PH1.EMO.GUIDE cannot grant authority, change gate order, or emit execution directives.
- Hard rule: PH1.EMO.GUIDE may read PH1.EMO.CORE references but cannot override PH1.EMO.CORE contract boundaries.

## Related Engine Boundaries
- PH1.X consumption is limited to response-tone shaping (no meaning change).
- PH1.TTS consumption is limited to rendering profile hints.
- PH1.PERSONA persistence remains separate; PH1.EMO.GUIDE itself has no runtime DB writes in this slice.
