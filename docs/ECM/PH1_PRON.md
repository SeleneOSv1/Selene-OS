# PH1_PRON ECM (Design vNext)

## Engine Header
- engine_id: PH1.PRON
- role: Pronunciation enrollment and lexicon-pack hints for speech engines
- placement: TURN_OPTIONAL

## Capability List

### capability_id: PRON_LEXICON_PACK_BUILD
- input_schema: bounded request envelope from Selene OS (`tenant_id`, optional `user_id`, `scope`, `consent_asserted`, pronunciation entries, `correlation_id`, `turn_id`)
- output_schema: `PronLexiconPackBuildOk` (pack_id + target_engines + bounded entries + `tenant_scoped=true` + `no_meaning_drift=true`)
- allowed_callers: SELENE_OS_ONLY
- side_effects: NONE
- failure_modes: INPUT_SCHEMA_INVALID, UPSTREAM_INPUT_MISSING, CONSENT_REQUIRED, BUDGET_EXCEEDED, INTERNAL_PIPELINE_ERROR
- reason_codes: PH1_PRON_INPUT_SCHEMA_INVALID, PH1_PRON_UPSTREAM_INPUT_MISSING, PH1_PRON_CONSENT_REQUIRED, PH1_PRON_BUDGET_EXCEEDED, PH1_PRON_INTERNAL_PIPELINE_ERROR

### capability_id: PRON_APPLY_VALIDATE
- input_schema: bounded self-check request (`pack_id`, `target_engine`, `locale_tag`, bounded entries)
- output_schema: `PronApplyValidateOk` (`validation_status`, bounded diagnostics, `no_meaning_drift=true`)
- allowed_callers: SELENE_OS_ONLY
- side_effects: NONE
- failure_modes: VALIDATION_FAILED, INPUT_SCHEMA_INVALID, UPSTREAM_INPUT_MISSING, BUDGET_EXCEEDED
- reason_codes: PH1_PRON_VALIDATION_FAILED, PH1_PRON_INPUT_SCHEMA_INVALID, PH1_PRON_UPSTREAM_INPUT_MISSING, PH1_PRON_BUDGET_EXCEEDED

## Constraints
- Engines never call engines directly; Selene OS orchestrates all sequencing.
- Non-authoritative assist engine; outputs are advisory only.
- No execution path and no authority mutation.
- Tenant-scope is mandatory; user-scope requires explicit consent.
- Integration boundary is fixed: PH1.PRON may feed pronunciation hints to PH1.TTS and robustness hints to PH1.VOICE.ID/PH1.W only.

## Related Engine Boundary (`PH1.KNOW`)
- PH1.KNOW and PH1.PRON remain separate capability surfaces:
  - PH1.KNOW: tenant dictionary + pronunciation-hint composition.
  - PH1.PRON: pronunciation enrollment + lexicon validation for speech engines.
- Selene OS may combine outputs only when each engine's validation status is `OK`; PH1.PRON capabilities must not absorb PH1.KNOW authority scope.
