# PH1_SEARCH ECM (Design vNext)

## Engine Header
- engine_id: PH1.SEARCH
- role: Read-only search + evidence-query grounding hints
- placement: TURN_OPTIONAL

## Capability List

### capability_id: SEARCH_PLAN_BUILD
- input_schema: bounded request envelope from Selene OS (engine-specific payload + correlation_id + turn_id)
- output_schema: `SearchPlanBuildOk` (no_intent_drift=true, bounded planned_queries, deterministic ordering)
- allowed_callers: SELENE_OS_ONLY
- side_effects: NONE
- failure_modes: INPUT_SCHEMA_INVALID, UPSTREAM_INPUT_MISSING, BUDGET_EXCEEDED, INTERNAL_PIPELINE_ERROR
- reason_codes: PH1_SEARCH_INPUT_SCHEMA_INVALID, PH1_SEARCH_UPSTREAM_INPUT_MISSING, PH1_SEARCH_BUDGET_EXCEEDED, PH1_SEARCH_INTERNAL_PIPELINE_ERROR

### capability_id: SEARCH_QUERY_REWRITE
- input_schema: bounded self-check request (engine payload + deterministic constraints)
- output_schema: `SearchQueryRewriteOk` (validation_status + rewritten_queries + bounded diagnostics, no_intent_drift=true)
- allowed_callers: SELENE_OS_ONLY
- side_effects: NONE
- failure_modes: VALIDATION_FAILED, INPUT_SCHEMA_INVALID, BUDGET_EXCEEDED
- reason_codes: PH1_SEARCH_VALIDATION_FAILED, PH1_SEARCH_INPUT_SCHEMA_INVALID, PH1_SEARCH_BUDGET_EXCEEDED

## Constraints
- Engines never call engines directly; Selene OS orchestrates all sequencing.
- Non-authoritative assist engine; outputs are advisory only.
- No execution path and no authority mutation.
- Hard no-intent-drift rule is mandatory: PH1.SEARCH may only produce query text; if rewritten text is not anchored to user query intent, validation must fail closed.
- Web evidence discipline: when PH1.SEARCH bundle includes ranked_source_ids and URL provenance, these fields must be preserved exactly for downstream PH1.CONTEXT composition.
