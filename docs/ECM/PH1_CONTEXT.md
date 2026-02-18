# PH1_CONTEXT ECM (Design vNext)

## Engine Header
- engine_id: PH1.CONTEXT
- role: Bounded context bundle assembly
- placement: ALWAYS_ON

## Capability List

### capability_id: CONTEXT_BUNDLE_BUILD
- input_schema: bounded request envelope from Selene OS (engine-specific payload + optional doc/summary/vision/web evidence bundles + optional KG fact bundles + correlation_id + turn_id)
- output_schema: bounded advisory payload with one selected item set + deterministic ordered context items + `preserved_evidence_refs=true`
- allowed_callers: SELENE_OS_ONLY
- side_effects: NONE
- failure_modes: INPUT_SCHEMA_INVALID, UPSTREAM_INPUT_MISSING, BUDGET_EXCEEDED, INTERNAL_PIPELINE_ERROR
- reason_codes: PH1_CONTEXT_OK_BUNDLE_BUILD, PH1_CONTEXT_INPUT_SCHEMA_INVALID, PH1_CONTEXT_UPSTREAM_INPUT_MISSING, PH1_CONTEXT_BUDGET_EXCEEDED, PH1_CONTEXT_INTERNAL_PIPELINE_ERROR

### capability_id: CONTEXT_BUNDLE_TRIM
- input_schema: bounded self-check request (engine payload + deterministic constraints)
- output_schema: validation_result (OK|FAIL) + bounded diagnostics + order/evidence preservation flags
- allowed_callers: SELENE_OS_ONLY
- side_effects: NONE
- failure_modes: VALIDATION_FAILED, INPUT_SCHEMA_INVALID, BUDGET_EXCEEDED
- reason_codes: PH1_CONTEXT_OK_BUNDLE_TRIM, PH1_CONTEXT_VALIDATION_FAILED, PH1_CONTEXT_INPUT_SCHEMA_INVALID, PH1_CONTEXT_BUDGET_EXCEEDED

## Constraints
- Engines never call engines directly; Selene OS orchestrates all sequencing.
- Non-authoritative assist engine; outputs are advisory only.
- No execution path and no authority mutation.
- Summary evidence discipline: when PH1.SUMMARY bundle is present, cited_evidence_ids must be preserved through context composition.
- Web evidence discipline: when PH1.SEARCH bundle is present, ranked_source_ids and URL provenance must be preserved through context composition.
- Multi-fusion discipline: when PH1.MULTI bundle is present, selected-signal integrity and ordered multimodal item ordering must be preserved through composition.
- KG discipline: when PH1.KG bundle is present, tenant scope and evidence refs must be preserved through composition.
- PH1.CONTEXT may consume PH1.RLL-derived retrieval scoring only from governance-approved ACTIVE artifacts.
- PH1.CONTEXT may consume PH1.CACHE hints only as advisory ranking metadata.
- `CONTEXT_BUNDLE_BUILD -> CONTEXT_BUNDLE_TRIM` is mandatory; Selene OS must fail closed when trim status is `FAIL`.

## Related Engine Boundary (salience ranking)
- PH1.CONTEXT performs salience/focus ranking internally as deterministic ordering metadata only.
- PH1.CONTEXT must not treat ranking metadata as evidence authority; source evidence bundles remain canonical.

## Related Engine Boundary (`PH1.MULTI`)
- PH1.CONTEXT may consume PH1.MULTI only after `MULTI_SIGNAL_ALIGN` passes (`validation_status=OK`).
- PH1.CONTEXT must fail closed if any multimodal vision/document item is missing `evidence_ref`.

## Related Engine Boundary (`PH1.RLL`)
- PH1.CONTEXT must not accept ungoverned PH1.RLL output at runtime.
- PH1.RLL context scoring proposals require governed activation before PH1.CONTEXT consumption.

## Related Engine Boundary (`PH1.CACHE`)
- PH1.CONTEXT may consume PH1.CACHE only after `CACHE_HINT_SNAPSHOT_REFRESH` returns `validation_status=OK`.
- PH1.CONTEXT must fail closed on cache-hint bundles that omit gate-preserving skeleton flags.

## Related Engine Boundary (`PH1.KG`)
- PH1.CONTEXT may consume PH1.KG only after `KG_FACT_BUNDLE_SELECT` returns `validation_status=OK`.
- PH1.CONTEXT must fail closed when KG bundles violate tenant scope or contain missing evidence refs.
