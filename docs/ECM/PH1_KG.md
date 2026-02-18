# PH1_KG ECM (Design vNext)

## Engine Header
- engine_id: PH1.KG
- role: Tenant-scoped knowledge-graph relationship grounding
- placement: TURN_OPTIONAL

## Capability List

### capability_id: KG_ENTITY_LINK
- input_schema:
  - bounded envelope (`correlation_id`, `turn_id`, `max_entity_candidates`, `max_fact_candidates`, `max_diagnostics`)
  - required `tenant_id`
  - bounded entity candidates (`candidate_id`, `tenant_id`, `entity_type`, `entity_key`, `canonical_label`, `confidence_bp`, `evidence_ref`)
  - bounded relation hints (`person_has_role`, `person_in_team`, `person_on_project`, `project_in_department`)
- output_schema:
  - one selected fact id
  - deterministic ordered fact candidates (`relation_type`, subject/object candidate ids, `priority_bp`, `evidence_ref`)
  - boundary flags: `tenant_scoped=true`, `evidence_backed=true`, `no_guessing=true`, `advisory_only=true`, `no_execution_authority=true`
- allowed_callers: SELENE_OS_ONLY
- side_effects: NONE
- failure_modes: INPUT_SCHEMA_INVALID, UPSTREAM_INPUT_MISSING, BUDGET_EXCEEDED, VALIDATION_FAILED, INTERNAL_PIPELINE_ERROR
- reason_codes:
  - PH1_KG_OK_ENTITY_LINK
  - PH1_KG_INPUT_SCHEMA_INVALID
  - PH1_KG_UPSTREAM_INPUT_MISSING
  - PH1_KG_BUDGET_EXCEEDED
  - PH1_KG_VALIDATION_FAILED
  - PH1_KG_INTERNAL_PIPELINE_ERROR

### capability_id: KG_FACT_BUNDLE_SELECT
- input_schema:
  - selected fact id + ordered fact candidates from `KG_ENTITY_LINK`
  - same bounded envelope constraints for deterministic replay
  - guard flags: `tenant_scope_required=true`, `evidence_required=true`, `no_guessing_required=true`
- output_schema:
  - validation_result (`OK|FAIL`)
  - bounded diagnostics
  - preservation flags (`preserved_tenant_scope`, `preserved_evidence_refs`, `no_guessing_confirmed`)
  - boundary flags: `advisory_only=true`, `no_execution_authority=true`
- allowed_callers: SELENE_OS_ONLY
- side_effects: NONE
- failure_modes: VALIDATION_FAILED, INPUT_SCHEMA_INVALID, UPSTREAM_INPUT_MISSING, BUDGET_EXCEEDED
- reason_codes:
  - PH1_KG_OK_FACT_BUNDLE_SELECT
  - PH1_KG_VALIDATION_FAILED
  - PH1_KG_INPUT_SCHEMA_INVALID
  - PH1_KG_UPSTREAM_INPUT_MISSING
  - PH1_KG_BUDGET_EXCEEDED

## Constraints
- Engines never call engines directly; Selene OS orchestrates all sequencing.
- Non-authoritative assist engine; outputs are advisory only.
- No execution path and no authority mutation.
- Cross-tenant relation composition is forbidden and must fail closed.
- Every fact candidate must be evidence-backed.
- Runtime no-guessing rule is mandatory (`KG_ENTITY_LINK -> KG_FACT_BUNDLE_SELECT` must preserve explicit evidence + tenant scope).

## Related Engine Boundaries
- PH1.CONTEXT and PH1.NLP may consume PH1.KG outputs only through Selene OS and only when `KG_FACT_BUNDLE_SELECT=OK`.
- PH1.KG may use PH1.KNOW-derived tenant dictionary anchors only as bounded seed metadata (`docs/DB_WIRING/PH1_KNOW.md` + `docs/ECM/PH1_KNOW.md`); it must not bypass evidence/no-guessing constraints.
