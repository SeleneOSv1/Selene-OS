# PH1_PRUNE ECM (Design vNext)

## Engine Header
- engine_id: PH1.PRUNE
- role: Missing-field pruning for one-question clarify discipline
- placement: TURN_OPTIONAL

## Capability List

### capability_id: PRUNE_MISSING_FIELDS
- input_schema: bounded request envelope from Selene OS (`correlation_id`, `turn_id`, `max_missing_fields`) + `required_fields_missing`, `ambiguity_flags`, `uncertain_field_hints`, optional `prefilled_fields`, `confirmed_fields`, and optional `previous_clarify_field`
- output_schema: `PruneMissingFieldsOk` (`selected_missing_field`, deterministic `ordered_missing_fields`, `no_execution_authority=true`)
- allowed_callers: SELENE_OS_ONLY
- side_effects: NONE
- failure_modes: INPUT_SCHEMA_INVALID, UPSTREAM_INPUT_MISSING, BUDGET_EXCEEDED, INTERNAL_PIPELINE_ERROR
- reason_codes: PH1_PRUNE_INPUT_SCHEMA_INVALID, PH1_PRUNE_UPSTREAM_INPUT_MISSING, PH1_PRUNE_BUDGET_EXCEEDED, PH1_PRUNE_INTERNAL_PIPELINE_ERROR

### capability_id: PRUNE_CLARIFY_ORDER
- input_schema: bounded self-check request (`required_fields_missing`, `selected_missing_field`, `ordered_missing_fields`, optional `previous_clarify_field`)
- output_schema: `PruneClarifyOrderOk` (`validation_status`, bounded diagnostics, `no_execution_authority=true`)
- allowed_callers: SELENE_OS_ONLY
- side_effects: NONE
- failure_modes: VALIDATION_FAILED, INPUT_SCHEMA_INVALID, UPSTREAM_INPUT_MISSING, BUDGET_EXCEEDED
- reason_codes: PH1_PRUNE_VALIDATION_FAILED, PH1_PRUNE_INPUT_SCHEMA_INVALID, PH1_PRUNE_UPSTREAM_INPUT_MISSING, PH1_PRUNE_BUDGET_EXCEEDED

## Constraints
- Engines never call engines directly; Selene OS orchestrates all sequencing.
- Non-authoritative assist engine; outputs are advisory only.
- No execution path and no authority mutation.
- PH1.PRUNE must output ASCII snake_case field keys only.
- PH1.PRUNE must not invent new fields; output field keys must be a subset of PH1.NLP `required_fields_missing`.
- PH1.PRUNE must emit exactly one selected field candidate for PH1.X clarify path.

## Integration Boundary (Related Engines)
- Upstream source of missing fields is PH1.NLP only.
- Downstream consumer is PH1.X clarify packet construction only.
- If PRUNE validation fails, Selene OS must fail closed and skip PRUNE-derived handoff.
- PH1.RLL-derived clarify-order tuning may be applied only from governance-approved ACTIVE artifacts.
