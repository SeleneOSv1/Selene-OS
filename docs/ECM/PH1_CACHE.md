# PH1_CACHE ECM (Design vNext)

## Engine Header
- engine_id: PH1.CACHE
- role: Cached decision-path skeleton management (advisory-only)
- placement: TURN_OPTIONAL

## Capability List

### capability_id: CACHE_HINT_SNAPSHOT_READ
- input_schema:
  - bounded envelope (`correlation_id`, `turn_id`, `max_skeletons`, `max_diagnostics`)
  - bounded planning refs (`intent_type`, `environment_profile_ref`, optional `persona_profile_ref`, optional route hint)
  - optional governed `cache_policy_pack_id`
- output_schema:
  - one selected skeleton id
  - deterministic ordered `CachePlanSkeleton` list
  - boundary flags: `advisory_only=true`, `no_execution_authority=true`
- allowed_callers: SELENE_OS_ONLY
- side_effects: NONE
- failure_modes: INPUT_SCHEMA_INVALID, UPSTREAM_INPUT_MISSING, BUDGET_EXCEEDED, POLICY_DISABLED, INTERNAL_PIPELINE_ERROR
- reason_codes:
  - PH1_CACHE_OK_HINT_SNAPSHOT_READ
  - PH1_CACHE_INPUT_SCHEMA_INVALID
  - PH1_CACHE_UPSTREAM_INPUT_MISSING
  - PH1_CACHE_BUDGET_EXCEEDED
  - PH1_CACHE_POLICY_DISABLED
  - PH1_CACHE_INTERNAL_PIPELINE_ERROR

### capability_id: CACHE_HINT_SNAPSHOT_REFRESH
- input_schema:
  - selected id + ordered skeleton list from READ step
  - same bounded planning refs for deterministic replay
  - governed-artifact guard flag (`contains_ungoverned_artifacts`)
- output_schema:
  - validation_result (`OK|FAIL`)
  - bounded diagnostics
  - governance flag (`all_artifacts_governed_active`)
  - boundary flags: `advisory_only=true`, `no_execution_authority=true`
- allowed_callers: SELENE_OS_ONLY
- side_effects: NONE
- failure_modes: VALIDATION_FAILED, INPUT_SCHEMA_INVALID, BUDGET_EXCEEDED, POLICY_DISABLED, UNGOVERNED_ARTIFACT
- reason_codes:
  - PH1_CACHE_OK_HINT_SNAPSHOT_REFRESH
  - PH1_CACHE_VALIDATION_FAILED
  - PH1_CACHE_INPUT_SCHEMA_INVALID
  - PH1_CACHE_BUDGET_EXCEEDED
  - PH1_CACHE_POLICY_DISABLED
  - PH1_CACHE_UNGOVERNED_ARTIFACT

## Constraints
- Engines never call engines directly; Selene OS orchestrates all sequencing.
- Non-authoritative assist engine; outputs are advisory only.
- No execution path and no authority mutation.
- Every cache skeleton must preserve gate boundaries (`requires_access_gate=true`, `requires_simulation_gate=true`).
- PH1.CACHE may consume PH1.RLL-derived optimization hints only from governance-approved ACTIVE artifacts.

## Related Engine Boundaries
- PH1.PAE -> PH1.CACHE link is advisory-only; PH1.CACHE refuses ungoverned artifact input.
- PH1.PAE hints are valid only from `PAE_ADAPTATION_HINT_EMIT` with `validation_status=OK`.
- PH1.PREFETCH and PH1.CONTEXT may consume PH1.CACHE output only through Selene OS curated forwarding.
- PH1.CONTEXT integration requires `CACHE_HINT_SNAPSHOT_REFRESH` validation status `OK`.
