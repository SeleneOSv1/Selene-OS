# PH1_LANG ECM (Design vNext)

## Engine Header
- engine_id: PH1.LANG
- role: Multilingual detection, segmentation, and response-language mapping
- placement: TURN_OPTIONAL

## Capability List

### capability_id: LANG_HINTS_BUILD
- input_schema: bounded request envelope from Selene OS (engine-specific payload + correlation_id + turn_id)
- output_schema: bounded advisory payload with deterministic ordering
- allowed_callers: SELENE_OS_ONLY
- side_effects: NONE
- failure_modes: INPUT_SCHEMA_INVALID, UPSTREAM_INPUT_MISSING, BUDGET_EXCEEDED, INTERNAL_PIPELINE_ERROR
- reason_codes: PH1_LANG_INPUT_SCHEMA_INVALID, PH1_LANG_UPSTREAM_INPUT_MISSING, PH1_LANG_BUDGET_EXCEEDED, PH1_LANG_INTERNAL_PIPELINE_ERROR

### capability_id: LANG_CONSISTENCY_CHECK
- input_schema: bounded self-check request (engine payload + deterministic constraints)
- output_schema: validation_result (OK|FAIL) + bounded diagnostics
- allowed_callers: SELENE_OS_ONLY
- side_effects: NONE
- failure_modes: VALIDATION_FAILED, INPUT_SCHEMA_INVALID, BUDGET_EXCEEDED
- reason_codes: PH1_LANG_VALIDATION_FAILED, PH1_LANG_INPUT_SCHEMA_INVALID, PH1_LANG_BUDGET_EXCEEDED

### capability_id: LANG_MULTIPLE_DETECT
- input_schema: transcript_text, locale_hint(optional), source_modality(VOICE|TEXT), correlation_id, turn_id
- output_schema: detected_languages[], segment_spans[] (start/end + language_tag), dominant_language_tag, reason_code
- allowed_callers: SELENE_OS_ONLY
- side_effects: NONE
- failure_modes: INPUT_SCHEMA_INVALID, SEGMENTATION_FAILED, BUDGET_EXCEEDED
- reason_codes: PH1_LANG_INPUT_SCHEMA_INVALID, PH1_LANG_SEGMENTATION_FAILED, PH1_LANG_BUDGET_EXCEEDED

### capability_id: LANG_SEGMENT_RESPONSE_MAP
- input_schema: detected_languages[], segment_spans[], user_language_preferences(optional), response_mode
- output_schema: response_language_plan[] (segment_or_turn -> language_tag), default_response_language, reason_code
- allowed_callers: SELENE_OS_ONLY
- side_effects: NONE
- failure_modes: INPUT_SCHEMA_INVALID, LANGUAGE_PLAN_CONFLICT, BUDGET_EXCEEDED
- reason_codes: PH1_LANG_INPUT_SCHEMA_INVALID, PH1_LANG_PLAN_CONFLICT, PH1_LANG_BUDGET_EXCEEDED

## Constraints
- Engines never call engines directly; Selene OS orchestrates all sequencing.
- Non-authoritative assist engine; outputs are advisory only.
- No execution path and no authority mutation.
- Runtime order for multilingual normalization is enforced by Selene OS: `PH1.LANG -> PH1.SRL -> PH1.NLP`.
