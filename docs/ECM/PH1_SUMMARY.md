# PH1_SUMMARY ECM (Design vNext)

## Engine Header
- engine_id: PH1.SUMMARY
- role: Evidence-backed summary synthesis for downstream context/understanding
- placement: TURN_OPTIONAL

## Capability List

### capability_id: SUMMARY_BUILD
- input_schema: bounded request envelope from Selene OS (evidence bundle + correlation_id + turn_id + max_summary_bullets)
- output_schema: bounded summary bullets with deterministic order and explicit cited_evidence_ids
- allowed_callers: SELENE_OS_ONLY
- side_effects: NONE
- failure_modes: INPUT_SCHEMA_INVALID, UPSTREAM_INPUT_MISSING, BUDGET_EXCEEDED, INTERNAL_PIPELINE_ERROR
- reason_codes: PH1_SUMMARY_INPUT_SCHEMA_INVALID, PH1_SUMMARY_UPSTREAM_INPUT_MISSING, PH1_SUMMARY_BUDGET_EXCEEDED, PH1_SUMMARY_INTERNAL_PIPELINE_ERROR

### capability_id: SUMMARY_CITATION_VALIDATE
- input_schema: bounded validation request (evidence bundle + summary bullets + deterministic constraints)
- output_schema: validation_result (OK|FAIL) + bounded diagnostics
- allowed_callers: SELENE_OS_ONLY
- side_effects: NONE
- failure_modes: VALIDATION_FAILED, INPUT_SCHEMA_INVALID, BUDGET_EXCEEDED
- reason_codes: PH1_SUMMARY_VALIDATION_FAILED, PH1_SUMMARY_INPUT_SCHEMA_INVALID, PH1_SUMMARY_BUDGET_EXCEEDED

## Constraints
- Engines never call engines directly; Selene OS orchestrates all sequencing.
- Non-authoritative assist engine; outputs are advisory only.
- No execution path and no authority mutation.
- Evidence discipline is mandatory: every summary bullet must cite at least one provided evidence item.
- No-inference rule is mandatory: citation validation must fail closed when bullet content is not grounded in provided evidence text.
- PH1.SUMMARY output is context input only through Selene OS; PH1.CONTEXT remains the canonical bounded context assembler.
