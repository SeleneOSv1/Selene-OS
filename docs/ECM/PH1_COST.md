# PH1_COST ECM (Design vNext)

## Engine Header
- engine_id: PH1.COST
- role: Unified turn-policy pacing + budget guardrails for runtime routes
- placement: TURN_OPTIONAL

## Capability List

### capability_id: COST_BUDGET_PLAN_BUILD
- input_schema: bounded request envelope from Selene OS (engine-specific payload + correlation_id + turn_id + per-user/day lane budgets)
- output_schema: `CostBudgetPlanBuildOk` (`budget_scope`, `route_guardrails[]`, urgency + delivery preference metadata, `no_truth_mutation=true`)
- allowed_callers: SELENE_OS_ONLY
- side_effects: NONE
- failure_modes: INPUT_SCHEMA_INVALID, UPSTREAM_INPUT_MISSING, BUDGET_EXCEEDED, INTERNAL_PIPELINE_ERROR
- reason_codes: PH1_COST_INPUT_SCHEMA_INVALID, PH1_COST_UPSTREAM_INPUT_MISSING, PH1_COST_BUDGET_EXCEEDED, PH1_COST_INTERNAL_PIPELINE_ERROR

### capability_id: COST_ROUTE_GUARD_VALIDATE
- input_schema: bounded self-check request (budget_scope + lane budgets + generated guardrails)
- output_schema: `CostRouteGuardValidateOk` (`validation_status`, bounded diagnostics, `no_truth_mutation=true`)
- allowed_callers: SELENE_OS_ONLY
- side_effects: NONE
- failure_modes: VALIDATION_FAILED, INPUT_SCHEMA_INVALID, BUDGET_EXCEEDED
- reason_codes: PH1_COST_VALIDATION_FAILED, PH1_COST_INPUT_SCHEMA_INVALID, PH1_COST_BUDGET_EXCEEDED

## Constraints
- Engines never call engines directly; Selene OS orchestrates all sequencing.
- Non-authoritative assist engine; outputs are advisory only.
- No execution path and no authority mutation.
- Hard no-autonomous-action rule is mandatory: urgency/delivery metadata can tune routing budgets and response pacing only; it cannot trigger actions.
- Hard rule: PH1.COST never changes truth; it only emits permissible routing/degrade hints.

## Related Engine Boundary (`PH1.QUOTA`)
- PH1.COST capabilities are advisory and must not be treated as execution gate decisions.
- Authoritative lane gating (`ALLOW | WAIT | REFUSE`) is owned by `PH1.QUOTA` and consumed by Selene OS after/beside PH1.COST hints.
