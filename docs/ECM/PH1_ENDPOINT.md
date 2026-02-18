# PH1_ENDPOINT ECM (Design vNext)

## Engine Header
- engine_id: PH1.ENDPOINT
- role: Endpoint boundary assist for capture/transcript alignment and turn-segmentation hints
- placement: TURN_OPTIONAL

## Capability List

### capability_id: ENDPOINT_HINTS_BUILD
- input_schema: bounded request envelope from Selene OS (`correlation_id`, `turn_id`, `max_vad_windows`) + ordered `vad_windows[]`, `transcript_token_estimate`, `tts_playback_active`, optional validated `listen_environment_profile_ref`
- output_schema: `EndpointHintsBuildOk` (`selected_segment_id`, ordered `segment_hints[]`, `no_semantic_mutation=true`, `no_execution_authority=true`)
- allowed_callers: SELENE_OS_ONLY
- side_effects: NONE
- failure_modes: INPUT_SCHEMA_INVALID, UPSTREAM_INPUT_MISSING, BUDGET_EXCEEDED, INTERNAL_PIPELINE_ERROR
- reason_codes: PH1_ENDPOINT_INPUT_SCHEMA_INVALID, PH1_ENDPOINT_UPSTREAM_INPUT_MISSING, PH1_ENDPOINT_BUDGET_EXCEEDED, PH1_ENDPOINT_INTERNAL_PIPELINE_ERROR

### capability_id: ENDPOINT_BOUNDARY_SCORE
- input_schema: bounded self-check request (`selected_segment_id`, ordered `segment_hints[]`, optional `previous_selected_segment_id`)
- output_schema: `EndpointBoundaryScoreOk` (`validation_status`, bounded diagnostics, `no_semantic_mutation=true`, `no_execution_authority=true`)
- allowed_callers: SELENE_OS_ONLY
- side_effects: NONE
- failure_modes: VALIDATION_FAILED, INPUT_SCHEMA_INVALID, UPSTREAM_INPUT_MISSING, BUDGET_EXCEEDED
- reason_codes: PH1_ENDPOINT_VALIDATION_FAILED, PH1_ENDPOINT_INPUT_SCHEMA_INVALID, PH1_ENDPOINT_UPSTREAM_INPUT_MISSING, PH1_ENDPOINT_BUDGET_EXCEEDED

## Constraints
- Engines never call engines directly; Selene OS orchestrates all sequencing.
- Non-authoritative assist engine; outputs are advisory only.
- No execution path and no authority mutation.
- PH1.ENDPOINT outputs are perception hints only; no transcript text mutation and no intent mutation are allowed.
- `selected_segment_id` must resolve to one segment in `ordered_segment_hints`.
- Validation capability must fail when ordered hints drift from selected-segment semantics.

## Integration Boundary (Related Engines)
- Upstream runtime window source is PH1.K signals only (via Selene OS).
- Optional PH1.LISTEN input is advisory-only and must be accepted only when LISTEN filter status is `OK`.
- Downstream transcript gate owner is PH1.C; PH1.ENDPOINT must not emit pass/reject gate decisions.
