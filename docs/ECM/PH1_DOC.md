# PH1_DOC ECM (Design vNext)

## Engine Header
- engine_id: PH1.DOC
- role: Document evidence extraction
- placement: TURN_OPTIONAL

## Capability List

### capability_id: DOC_EVIDENCE_EXTRACT
- input_schema: bounded request envelope from Selene OS (engine-specific payload + correlation_id + turn_id)
- output_schema: `DocEvidenceExtractOk` (evidence_backed_only=true, bounded evidence_items, deterministic ordering)
- allowed_callers: SELENE_OS_ONLY
- side_effects: NONE
- failure_modes: INPUT_SCHEMA_INVALID, UPSTREAM_INPUT_MISSING, BUDGET_EXCEEDED, INTERNAL_PIPELINE_ERROR
- reason_codes: PH1_DOC_INPUT_SCHEMA_INVALID, PH1_DOC_UPSTREAM_INPUT_MISSING, PH1_DOC_BUDGET_EXCEEDED, PH1_DOC_INTERNAL_PIPELINE_ERROR

### capability_id: DOC_CITATION_MAP_BUILD
- input_schema: bounded self-check request (engine payload + deterministic constraints)
- output_schema: `DocCitationMapBuildOk` (validation_status + bounded diagnostics, no side effects)
- allowed_callers: SELENE_OS_ONLY
- side_effects: NONE
- failure_modes: VALIDATION_FAILED, INPUT_SCHEMA_INVALID, BUDGET_EXCEEDED
- reason_codes: PH1_DOC_VALIDATION_FAILED, PH1_DOC_INPUT_SCHEMA_INVALID, PH1_DOC_BUDGET_EXCEEDED

## Constraints
- Engines never call engines directly; Selene OS orchestrates all sequencing.
- Non-authoritative assist engine; outputs are advisory only.
- No execution path and no authority mutation.
- Read-only analyzer constraint: this engine transforms/extracts from provided evidence only; PH1.E is the only tool execution engine.
- No-inference rule is mandatory: citation validation must fail closed when snippet text is not grounded in cited evidence text.
- Downstream summary synthesis is optional and OS-owned: PH1.DOC outputs may be routed to PH1.SUMMARY before PH1.CONTEXT/PH1.NLP.
- Context composition ownership is OS+PH1.CONTEXT: PH1.DOC outputs are advisory evidence input only.
