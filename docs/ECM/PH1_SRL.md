# PH1_SRL ECM (Design vNext)

## Engine Header
- engine_id: PH1.SRL
- role: Post-STT semantic repair and deterministic argument normalization
- placement: ALWAYS_ON

## Capability List

### capability_id: SRL_FRAME_BUILD
- input_schema:
  - bounded envelope (`correlation_id`, `turn_id`, `max_spans`, `max_notes`, `max_ambiguities`, `max_diagnostics`)
  - `transcript_hash`, `transcript_text`, `language_tag`
  - bounded uncertain spans and bounded PH1.KNOW dictionary hints
  - hard boundary flags (`normalize_shorthand=true`, `preserve_code_switch=true`, `no_translate=true`)
- output_schema:
  - `repaired_transcript_text` (verbatim-preserving)
  - deterministic `frame_spans` (`raw_text`, `normalized_text`, `language_tag`, `role_label`)
  - bounded `repair_notes`, bounded `ambiguity_flags`
  - boundary flags: `preserve_code_switch=true`, `no_new_facts=true`, `no_translation_performed=true`
- allowed_callers: SELENE_OS_ONLY
- side_effects: NONE
- failure_modes: INPUT_SCHEMA_INVALID, UPSTREAM_INPUT_MISSING, BUDGET_EXCEEDED, INTERNAL_PIPELINE_ERROR
- reason_codes:
  - PH1_SRL_OK_FRAME_BUILD
  - PH1_SRL_INPUT_SCHEMA_INVALID
  - PH1_SRL_UPSTREAM_INPUT_MISSING
  - PH1_SRL_BUDGET_EXCEEDED
  - PH1_SRL_INTERNAL_PIPELINE_ERROR

### capability_id: SRL_ARGUMENT_NORMALIZE
- input_schema:
  - bounded envelope + `transcript_hash`
  - `repaired_transcript_text`, `frame_spans`, `repair_notes`, `ambiguity_flags`
  - hard boundary flags (`no_intent_change_required=true`, `no_fact_invention_required=true`, `preserve_code_switch=true`, `no_translate=true`)
- output_schema:
  - `validation_status (OK|FAIL)` + bounded diagnostics
  - deterministic `normalized_frame_spans`
  - bounded `ambiguity_flags` + `clarify_required`
  - boundary flags: `preserve_code_switch=true`, `no_new_facts=true`, `no_translation_performed=true`, `no_intent_change=true`
- allowed_callers: SELENE_OS_ONLY
- side_effects: NONE
- failure_modes: INPUT_SCHEMA_INVALID, UPSTREAM_INPUT_MISSING, BUDGET_EXCEEDED, VALIDATION_FAILED, INTERNAL_PIPELINE_ERROR
- reason_codes:
  - PH1_SRL_OK_ARGUMENT_NORMALIZE
  - PH1_SRL_INPUT_SCHEMA_INVALID
  - PH1_SRL_UPSTREAM_INPUT_MISSING
  - PH1_SRL_BUDGET_EXCEEDED
  - PH1_SRL_VALIDATION_FAILED
  - PH1_SRL_INTERNAL_PIPELINE_ERROR

## Constraints
- Engines never call engines directly; Selene OS orchestrates all sequencing.
- Non-authoritative assist engine; outputs are advisory only.
- No execution path and no authority mutation.
- Hard no-translation/no-intent-drift boundary is mandatory.
- Ambiguity flags require `clarify_required=true`; SRL never resolves ambiguity by guessing.

## Related Engine Boundaries
- PH1.LANG may provide segmentation hints; PH1.SRL remains deterministic owner of repair output.
- PH1.KNOW may provide tenant-scoped dictionary hints; PH1.SRL treats them as advisory only.
- PH1.NLP consumes SRL output and remains final deterministic owner of understanding decisions.
- SRL ambiguity notes are consumed only by PH1.NLP deterministic clarify flow.
