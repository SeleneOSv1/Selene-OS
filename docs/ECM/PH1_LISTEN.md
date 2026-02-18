# PH1_LISTEN ECM (Design vNext)

## Engine Header
- engine_id: PH1.LISTEN
- role: Active listening environment classification and deterministic adaptation hint emission
- placement: TURN_OPTIONAL

## Capability List

### capability_id: LISTEN_SIGNAL_COLLECT
- input_schema:
  - bounded envelope (`correlation_id`, `turn_id`, `max_signal_windows`, `max_adjustments`, `max_diagnostics`)
  - bounded PH1.K signal windows (`window_id`, `source_engine`, `vad_confidence_bp`, `speech_likeness_bp`, `noise_level_dbfs`, overlap/silence timing, `evidence_ref`)
  - bounded correction/session context snapshots (`user_correction_count`, `delivery_switch_count`, `barge_in_count`, `correction_rate_bp`, meeting/car/privacy/text-preference flags)
- output_schema:
  - `environment_profile_ref`
  - one selected adjustment id + deterministic ordered adjustment hints (`capture_profile`, `endpoint_profile`, `delivery_policy_hint`, `priority_bp`)
  - boundary flags: `affects_capture_only=true`, `affects_delivery_mode_only=true`, `no_meaning_mutation=true`, `advisory_only=true`, `no_execution_authority=true`
- allowed_callers: SELENE_OS_ONLY
- side_effects: NONE
- failure_modes: INPUT_SCHEMA_INVALID, UPSTREAM_INPUT_MISSING, BUDGET_EXCEEDED, VALIDATION_FAILED, INTERNAL_PIPELINE_ERROR
- reason_codes:
  - PH1_LISTEN_OK_SIGNAL_COLLECT
  - PH1_LISTEN_INPUT_SCHEMA_INVALID
  - PH1_LISTEN_UPSTREAM_INPUT_MISSING
  - PH1_LISTEN_BUDGET_EXCEEDED
  - PH1_LISTEN_VALIDATION_FAILED
  - PH1_LISTEN_INTERNAL_PIPELINE_ERROR

### capability_id: LISTEN_SIGNAL_FILTER
- input_schema:
  - selected adjustment id + ordered adjustments from `LISTEN_SIGNAL_COLLECT`
  - `environment_profile_ref`
  - deterministic replay constraints from the same bounded envelope
  - `no_meaning_mutation_required=true`
- output_schema:
  - validation_result (`OK|FAIL`)
  - bounded diagnostics
  - apply flags (`applies_capture_profile`, `applies_endpoint_profile`, `applies_delivery_policy_hint`)
  - boundary flags: `no_meaning_mutation=true`, `advisory_only=true`, `no_execution_authority=true`
- allowed_callers: SELENE_OS_ONLY
- side_effects: NONE
- failure_modes: VALIDATION_FAILED, INPUT_SCHEMA_INVALID, UPSTREAM_INPUT_MISSING, BUDGET_EXCEEDED
- reason_codes:
  - PH1_LISTEN_OK_SIGNAL_FILTER
  - PH1_LISTEN_VALIDATION_FAILED
  - PH1_LISTEN_INPUT_SCHEMA_INVALID
  - PH1_LISTEN_UPSTREAM_INPUT_MISSING
  - PH1_LISTEN_BUDGET_EXCEEDED

## Constraints
- Engines never call engines directly; Selene OS orchestrates all sequencing.
- Non-authoritative assist engine; outputs are advisory only.
- No execution path and no authority mutation.
- Hard rule: LISTEN outputs may tune capture/endpointing/delivery mode hints only; transcript semantics and intent meaning must never be mutated.

## Related Engine Boundaries
- PH1.ENDPOINT/PH1.C may receive LISTEN hints only through Selene OS and only when `LISTEN_SIGNAL_FILTER=OK`.
- PH1.PAE and PH1.MULTI consume LISTEN output only as bounded advisory metadata.
- PH1.PAE consumption path is restricted to `PAE_POLICY_SCORE_BUILD` input context and cannot trigger execution directives.
