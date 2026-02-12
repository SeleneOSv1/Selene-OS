# PH1_PATTERN ECM (Design vNext)

## Engine Header
- engine_id: PH1.PATTERN
- role: Offline pattern mining for artifact proposals
- placement: OFFLINE_ONLY

## Capability List

### capability_id: PATTERN_MINE_OFFLINE
- input_schema: bounded request envelope from Selene OS (engine-specific payload + correlation_id + turn_id)
- output_schema: bounded advisory payload with deterministic ordering
- allowed_callers: OFFLINE_PIPELINE_ONLY
- side_effects: NONE
- failure_modes: INPUT_SCHEMA_INVALID, UPSTREAM_INPUT_MISSING, BUDGET_EXCEEDED, INTERNAL_PIPELINE_ERROR
- reason_codes: PH1_PATTERN_INPUT_SCHEMA_INVALID, PH1_PATTERN_UPSTREAM_INPUT_MISSING, PH1_PATTERN_BUDGET_EXCEEDED, PH1_PATTERN_INTERNAL_PIPELINE_ERROR

### capability_id: PATTERN_PROPOSAL_EMIT
- input_schema: bounded self-check request (engine payload + deterministic constraints)
- output_schema: validation_result (OK|FAIL) + bounded diagnostics
- allowed_callers: OFFLINE_PIPELINE_ONLY
- side_effects: NONE
- failure_modes: VALIDATION_FAILED, INPUT_SCHEMA_INVALID, BUDGET_EXCEEDED
- reason_codes: PH1_PATTERN_VALIDATION_FAILED, PH1_PATTERN_INPUT_SCHEMA_INVALID, PH1_PATTERN_BUDGET_EXCEEDED

## Constraints
- Engines never call engines directly; Selene OS orchestrates all sequencing.
- Non-authoritative assist engine; outputs are advisory only.
- No execution path and no authority mutation.
- Offline-only constraint: callable only by OFFLINE_PIPELINE_ONLY and never in-turn.
