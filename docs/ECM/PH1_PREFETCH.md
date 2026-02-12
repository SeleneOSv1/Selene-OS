# PH1_PREFETCH ECM (Design vNext)

## Engine Header
- engine_id: PH1.PREFETCH
- role: Prefetch candidate suggestion
- placement: TURN_OPTIONAL

## Capability List

### capability_id: PREFETCH_PLAN_BUILD
- input_schema: bounded request envelope from Selene OS (engine-specific payload + correlation_id + turn_id)
- output_schema: bounded advisory payload with deterministic ordering
- allowed_callers: SELENE_OS_ONLY
- side_effects: NONE
- failure_modes: INPUT_SCHEMA_INVALID, UPSTREAM_INPUT_MISSING, BUDGET_EXCEEDED, INTERNAL_PIPELINE_ERROR
- reason_codes: PH1_PREFETCH_INPUT_SCHEMA_INVALID, PH1_PREFETCH_UPSTREAM_INPUT_MISSING, PH1_PREFETCH_BUDGET_EXCEEDED, PH1_PREFETCH_INTERNAL_PIPELINE_ERROR

### capability_id: PREFETCH_PRIORITIZE
- input_schema: bounded self-check request (engine payload + deterministic constraints)
- output_schema: validation_result (OK|FAIL) + bounded diagnostics
- allowed_callers: SELENE_OS_ONLY
- side_effects: NONE
- failure_modes: VALIDATION_FAILED, INPUT_SCHEMA_INVALID, BUDGET_EXCEEDED
- reason_codes: PH1_PREFETCH_VALIDATION_FAILED, PH1_PREFETCH_INPUT_SCHEMA_INVALID, PH1_PREFETCH_BUDGET_EXCEEDED

## Constraints
- Engines never call engines directly; Selene OS orchestrates all sequencing.
- Non-authoritative assist engine; outputs are advisory only.
- No execution path and no authority mutation.
