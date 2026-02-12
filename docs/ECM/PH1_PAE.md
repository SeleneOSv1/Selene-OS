# PH1_PAE ECM (Design vNext)

## Engine Header
- engine_id: PH1.PAE
- role: Policy adaptation evaluation
- placement: TURN_OPTIONAL

## Capability List

### capability_id: PAE_POLICY_SCORE_BUILD
- input_schema: bounded request envelope from Selene OS (engine-specific payload + correlation_id + turn_id)
- output_schema: bounded advisory payload with deterministic ordering
- allowed_callers: SELENE_OS_ONLY
- side_effects: NONE
- failure_modes: INPUT_SCHEMA_INVALID, UPSTREAM_INPUT_MISSING, BUDGET_EXCEEDED, INTERNAL_PIPELINE_ERROR
- reason_codes: PH1_PAE_INPUT_SCHEMA_INVALID, PH1_PAE_UPSTREAM_INPUT_MISSING, PH1_PAE_BUDGET_EXCEEDED, PH1_PAE_INTERNAL_PIPELINE_ERROR

### capability_id: PAE_ADAPTATION_HINT_EMIT
- input_schema: bounded self-check request (engine payload + deterministic constraints)
- output_schema: validation_result (OK|FAIL) + bounded diagnostics
- allowed_callers: SELENE_OS_ONLY
- side_effects: NONE
- failure_modes: VALIDATION_FAILED, INPUT_SCHEMA_INVALID, BUDGET_EXCEEDED
- reason_codes: PH1_PAE_VALIDATION_FAILED, PH1_PAE_INPUT_SCHEMA_INVALID, PH1_PAE_BUDGET_EXCEEDED

## Constraints
- Engines never call engines directly; Selene OS orchestrates all sequencing.
- Non-authoritative assist engine; outputs are advisory only.
- No execution path and no authority mutation.
