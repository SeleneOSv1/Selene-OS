# PH1_DIAG ECM (Design vNext)

## Engine Header
- engine_id: PH1.DIAG
- role: Final pre-directive consistency checks
- placement: TURN_OPTIONAL

## Capability List

### capability_id: DIAG_CONSISTENCY_CHECK
- input_schema: bounded request envelope from Selene OS (engine-specific payload + correlation_id + turn_id)
- output_schema: `DiagConsistencyCheckOk` (diagnostic_flags + `no_execution_authority=true`)
- allowed_callers: SELENE_OS_ONLY
- side_effects: NONE
- failure_modes: INPUT_SCHEMA_INVALID, UPSTREAM_INPUT_MISSING, BUDGET_EXCEEDED, INTERNAL_PIPELINE_ERROR
- reason_codes: PH1_DIAG_INPUT_SCHEMA_INVALID, PH1_DIAG_UPSTREAM_INPUT_MISSING, PH1_DIAG_BUDGET_EXCEEDED, PH1_DIAG_INTERNAL_PIPELINE_ERROR

### capability_id: DIAG_REASON_SET_BUILD
- input_schema: bounded self-check request (engine payload + deterministic constraints)
- output_schema: `DiagReasonSetBuildOk` (validation_status + reason_set + bounded diagnostics + `no_execution_authority=true`)
- allowed_callers: SELENE_OS_ONLY
- side_effects: NONE
- failure_modes: VALIDATION_FAILED, INPUT_SCHEMA_INVALID, BUDGET_EXCEEDED
- reason_codes: PH1_DIAG_VALIDATION_FAILED, PH1_DIAG_INPUT_SCHEMA_INVALID, PH1_DIAG_BUDGET_EXCEEDED

## Constraints
- Engines never call engines directly; Selene OS orchestrates all sequencing.
- Non-authoritative assist engine; outputs are advisory only.
- No execution path and no authority mutation.
- Hard rule: PH1.DIAG can block/clarify only; it cannot invent facts or execute actions.
