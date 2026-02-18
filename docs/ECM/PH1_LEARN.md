# PH1_LEARN ECM (Design vNext)

## Engine Header
- engine_id: PH1.LEARN
- role: Learning signal aggregation and adaptation artifact package building
- placement: TURN_OPTIONAL (post-turn async)

## Capability List

### capability_id: LEARN_SIGNAL_AGGREGATE
- input_schema:
  - bounded envelope (`correlation_id`, `turn_id`, `max_signals`, `max_artifacts`, `max_diagnostics`)
  - required `tenant_id`
  - bounded signal list (`signal_id`, `signal_type`, `scope_hint`, `scope_ref`, `metric_key`, `metric_value_bp`, `occurrence_count`, consent/sensitivity flags, `evidence_ref`)
  - hard boundary flags (`require_derived_only_global=true`, `no_runtime_drift_required=true`)
- output_schema:
  - deterministic `selected_artifact_id`
  - deterministic ordered artifact candidates (`artifact_id`, target, scope, version, expected_effect_bp, provenance_ref, rollback_to)
  - boundary flags: `consent_safe=true`, `derived_only_global_preserved=true`, `advisory_only=true`, `no_execution_authority=true`
- allowed_callers: SELENE_OS_ONLY
- side_effects: NONE
- failure_modes: INPUT_SCHEMA_INVALID, UPSTREAM_INPUT_MISSING, BUDGET_EXCEEDED, CONSENT_REQUIRED, DERIVED_ONLY_GLOBAL_REQUIRED, INTERNAL_PIPELINE_ERROR
- reason_codes:
  - PH1_LEARN_OK_SIGNAL_AGGREGATE
  - PH1_LEARN_INPUT_SCHEMA_INVALID
  - PH1_LEARN_UPSTREAM_INPUT_MISSING
  - PH1_LEARN_BUDGET_EXCEEDED
  - PH1_LEARN_CONSENT_REQUIRED
  - PH1_LEARN_DERIVED_ONLY_GLOBAL_REQUIRED
  - PH1_LEARN_INTERNAL_PIPELINE_ERROR

### capability_id: LEARN_ARTIFACT_PACKAGE_BUILD
- input_schema:
  - bounded envelope (`correlation_id`, `turn_id`, `max_signals`, `max_artifacts`, `max_diagnostics`)
  - required `tenant_id`
  - `selected_artifact_id` + ordered artifact candidates from `LEARN_SIGNAL_AGGREGATE`
  - bounded `target_engines`
  - hard boundary flags (`require_versioning=true`, `require_rollback_ptr=true`, `no_runtime_drift_required=true`)
- output_schema:
  - `validation_status (OK|FAIL)`
  - bounded diagnostics
  - selected target engines
  - package guarantees (`artifacts_versioned`, `rollbackable`, `no_runtime_drift`)
  - boundary flags: `advisory_only=true`, `no_execution_authority=true`
- allowed_callers: SELENE_OS_ONLY
- side_effects: NONE
- failure_modes: INPUT_SCHEMA_INVALID, UPSTREAM_INPUT_MISSING, BUDGET_EXCEEDED, VALIDATION_FAILED, INTERNAL_PIPELINE_ERROR
- reason_codes:
  - PH1_LEARN_OK_ARTIFACT_PACKAGE_BUILD
  - PH1_LEARN_INPUT_SCHEMA_INVALID
  - PH1_LEARN_UPSTREAM_INPUT_MISSING
  - PH1_LEARN_BUDGET_EXCEEDED
  - PH1_LEARN_VALIDATION_FAILED
  - PH1_LEARN_INTERNAL_PIPELINE_ERROR

## Constraints
- Engines never call engines directly; Selene OS orchestrates all sequencing.
- Non-authoritative assist engine; outputs are advisory only.
- No execution path and no authority mutation.
- Consent and derived-only-global boundaries are mandatory and fail closed on violation.
- LEARN output is package proposal metadata; governance activation is outside PH1.LEARN runtime scope.

## Related Engine Boundaries
- PH1.FEEDBACK and PH1.LISTEN provide LEARN inputs only through Selene OS validated bundles.
- PH1.PERSONA/PH1.PAE/PH1.KNOW/PH1.CACHE/PH1.PRUNE/PH1.SEARCH may consume LEARN outputs only after Selene OS validation (`LEARN_ARTIFACT_PACKAGE_BUILD=OK`).
- PH1.PAE link is runtime-ordered: LEARN signals can influence `PAE_POLICY_SCORE_BUILD`; `PAE_ADAPTATION_HINT_EMIT` remains downstream validation/handoff only.
- PH1.PATTERN/PH1.RLL remain offline-only engines; PH1.LEARN consumes governed artifact outputs only, never direct runtime calls.
