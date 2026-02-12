# PH1_SEARCH ECM (Design vNext)

## Engine Header
- engine_id: PH1.SEARCH
- role: Read-only search planning hints
- placement: TURN_OPTIONAL

## Capability List

### capability_id: SEARCH_PLAN_BUILD
- input_schema: bounded request envelope from Selene OS (engine-specific payload + correlation_id + turn_id)
- output_schema: bounded advisory payload with deterministic ordering
- allowed_callers: SELENE_OS_ONLY
- side_effects: NONE
- failure_modes: INPUT_SCHEMA_INVALID, UPSTREAM_INPUT_MISSING, BUDGET_EXCEEDED, INTERNAL_PIPELINE_ERROR
- reason_codes: PH1_SEARCH_INPUT_SCHEMA_INVALID, PH1_SEARCH_UPSTREAM_INPUT_MISSING, PH1_SEARCH_BUDGET_EXCEEDED, PH1_SEARCH_INTERNAL_PIPELINE_ERROR

### capability_id: SEARCH_QUERY_REWRITE
- input_schema: bounded self-check request (engine payload + deterministic constraints)
- output_schema: validation_result (OK|FAIL) + bounded diagnostics
- allowed_callers: SELENE_OS_ONLY
- side_effects: NONE
- failure_modes: VALIDATION_FAILED, INPUT_SCHEMA_INVALID, BUDGET_EXCEEDED
- reason_codes: PH1_SEARCH_VALIDATION_FAILED, PH1_SEARCH_INPUT_SCHEMA_INVALID, PH1_SEARCH_BUDGET_EXCEEDED

## Constraints
- Engines never call engines directly; Selene OS orchestrates all sequencing.
- Non-authoritative assist engine; outputs are advisory only.
- No execution path and no authority mutation.
