# PH1_LANG ECM (Design vNext)

## Engine Header
- engine_id: PH1.LANG
- role: Multilingual detection/segmentation and response-language mapping
- placement: TURN_OPTIONAL

## Capability List

### capability_id: LANG_MULTIPLE_DETECT
- input_schema: bounded request envelope from Selene OS (`transcript_text`, optional `locale_hint`, `source_modality`, `correlation_id`, `turn_id`)
- output_schema: `LangMultipleDetectOk` (detected_languages + segment_spans + dominant_language_tag + `no_translation_performed=true`)
- allowed_callers: SELENE_OS_ONLY
- side_effects: NONE
- failure_modes: INPUT_SCHEMA_INVALID, UPSTREAM_INPUT_MISSING, SEGMENTATION_FAILED, BUDGET_EXCEEDED, INTERNAL_PIPELINE_ERROR
- reason_codes: PH1_LANG_INPUT_SCHEMA_INVALID, PH1_LANG_UPSTREAM_INPUT_MISSING, PH1_LANG_SEGMENTATION_FAILED, PH1_LANG_BUDGET_EXCEEDED, PH1_LANG_INTERNAL_PIPELINE_ERROR

### capability_id: LANG_SEGMENT_RESPONSE_MAP
- input_schema: bounded self-check request (`detected_languages`, `segment_spans`, optional user language preferences, `response_mode`)
- output_schema: `LangSegmentResponseMapOk` (`validation_status`, `response_language_plan`, `default_response_language`, bounded diagnostics, `no_translation_performed=true`)
- allowed_callers: SELENE_OS_ONLY
- side_effects: NONE
- failure_modes: VALIDATION_FAILED, INPUT_SCHEMA_INVALID, UPSTREAM_INPUT_MISSING, PLAN_CONFLICT, BUDGET_EXCEEDED
- reason_codes: PH1_LANG_VALIDATION_FAILED, PH1_LANG_INPUT_SCHEMA_INVALID, PH1_LANG_UPSTREAM_INPUT_MISSING, PH1_LANG_PLAN_CONFLICT, PH1_LANG_BUDGET_EXCEEDED

## Constraints
- Engines never call engines directly; Selene OS orchestrates all sequencing.
- Non-authoritative assist engine; outputs are advisory only.
- No execution path and no authority mutation.
- Hard no-translation rule is mandatory: PH1.LANG may only detect and map language boundaries; translation/rewriting is out of scope.
- Runtime order lock for multilingual normalization: `PH1.LANG -> PH1.SRL -> PH1.NLP`.
