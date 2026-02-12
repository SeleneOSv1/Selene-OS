# PH1_VISION ECM (Design vNext)

## Engine Header
- engine_id: PH1.VISION
- role: Image evidence extraction
- placement: TURN_OPTIONAL

## Capability List

### capability_id: VISION_EVIDENCE_EXTRACT
- input_schema: bounded request envelope from Selene OS (engine-specific payload + correlation_id + turn_id)
- output_schema: bounded advisory payload with deterministic ordering
- allowed_callers: SELENE_OS_ONLY
- side_effects: NONE
- failure_modes: INPUT_SCHEMA_INVALID, UPSTREAM_INPUT_MISSING, BUDGET_EXCEEDED, INTERNAL_PIPELINE_ERROR
- reason_codes: PH1_VISION_INPUT_SCHEMA_INVALID, PH1_VISION_UPSTREAM_INPUT_MISSING, PH1_VISION_BUDGET_EXCEEDED, PH1_VISION_INTERNAL_PIPELINE_ERROR

### capability_id: VISION_LAYOUT_SUMMARY
- input_schema: bounded self-check request (engine payload + deterministic constraints)
- output_schema: validation_result (OK|FAIL) + bounded diagnostics
- allowed_callers: SELENE_OS_ONLY
- side_effects: NONE
- failure_modes: VALIDATION_FAILED, INPUT_SCHEMA_INVALID, BUDGET_EXCEEDED
- reason_codes: PH1_VISION_VALIDATION_FAILED, PH1_VISION_INPUT_SCHEMA_INVALID, PH1_VISION_BUDGET_EXCEEDED

## Constraints
- Engines never call engines directly; Selene OS orchestrates all sequencing.
- Non-authoritative assist engine; outputs are advisory only.
- No execution path and no authority mutation.
- Read-only analyzer constraint: this engine transforms/extracts from provided evidence only; PH1.E is the only tool execution engine.
