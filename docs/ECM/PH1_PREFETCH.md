# PH1_PREFETCH ECM (Design vNext)

## Engine Header
- engine_id: PH1.PREFETCH
- role: Read-only prefetch/cache warmer hinting
- placement: TURN_OPTIONAL

## Capability List

### capability_id: PREFETCH_PLAN_BUILD
- input_schema: bounded request envelope from Selene OS (engine-specific payload + correlation_id + turn_id + intent/search/policy hints)
- output_schema: `PrefetchPlanBuildOk` (candidate list with deterministic id/ttl/rank/idempotency key + `read_only_only=true`)
- allowed_callers: SELENE_OS_ONLY
- side_effects: NONE
- failure_modes: INPUT_SCHEMA_INVALID, UPSTREAM_INPUT_MISSING, BUDGET_EXCEEDED, POLICY_DISABLED, INTERNAL_PIPELINE_ERROR
- reason_codes: PH1_PREFETCH_INPUT_SCHEMA_INVALID, PH1_PREFETCH_UPSTREAM_INPUT_MISSING, PH1_PREFETCH_BUDGET_EXCEEDED, PH1_PREFETCH_POLICY_DISABLED, PH1_PREFETCH_INTERNAL_PIPELINE_ERROR

### capability_id: PREFETCH_PRIORITIZE
- input_schema: bounded self-check request (intent/search context + candidate list from PREFETCH_PLAN_BUILD)
- output_schema: `PrefetchPrioritizeOk` (validation_status + prioritized_candidate_ids + bounded diagnostics + `read_only_only=true`)
- allowed_callers: SELENE_OS_ONLY
- side_effects: NONE
- failure_modes: VALIDATION_FAILED, INPUT_SCHEMA_INVALID, BUDGET_EXCEEDED, POLICY_DISABLED
- reason_codes: PH1_PREFETCH_VALIDATION_FAILED, PH1_PREFETCH_INPUT_SCHEMA_INVALID, PH1_PREFETCH_BUDGET_EXCEEDED, PH1_PREFETCH_POLICY_DISABLED

## Constraints
- Engines never call engines directly; Selene OS orchestrates all sequencing.
- Non-authoritative assist engine; outputs are advisory only.
- No execution path and no authority mutation.
- Read-only enforcement is mandatory (`read_only_only=true` in all OK outputs).
- PH1.PREFETCH may consume PH1.RLL-derived heuristics only after governed artifact activation.
- PH1.PREFETCH may consume PH1.CACHE hints only as bounded ordering metadata; PH1.CACHE never authorizes execution.
