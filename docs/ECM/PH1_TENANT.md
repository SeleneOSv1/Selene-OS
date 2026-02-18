# PH1_TENANT ECM (Design vNext)

## Engine Header
- engine_id: PH1.TENANT
- role: deterministic tenant and policy-context resolver for enterprise orchestration
- placement: ENTERPRISE_SUPPORT

## Capability List

### capability_id: TENANT_POLICY_EVALUATE
- input_schema:
  - bounded envelope (`correlation_id`, `turn_id`, `max_candidates`, `max_missing_fields`, `max_diagnostics`)
  - identity/session context (`identity_context`, optional `device_id`, optional `session_id`, `now`)
  - optional explicit selection (`explicit_tenant_selection_token`, optional `explicit_tenant_id`)
  - candidate tenant bindings (`tenant_id`, `policy_context_ref`, optional locale, disabled/policy-block flags)
  - invariants (`deterministic=true`, `no_permission_decision=true`, `no_cross_tenant_access=true`)
- output_schema:
  - deterministic policy output (`identity_known`, `candidate_count`, optional `selected_tenant_id`, `selection_source`, `multiple_match`)
  - reason-coded policy result
  - invariants (`deterministic=true`, `no_permission_decision=true`, `no_cross_tenant_access=true`)
- allowed_callers: SELENE_OS_ONLY
- side_effects: NONE
- failure_modes: INPUT_SCHEMA_INVALID, UPSTREAM_INPUT_MISSING, BUDGET_EXCEEDED, INTERNAL_PIPELINE_ERROR
- reason_codes:
  - PH1_TENANT_OK_POLICY_EVALUATE
  - PH1_TENANT_INPUT_SCHEMA_INVALID
  - PH1_TENANT_UPSTREAM_INPUT_MISSING
  - PH1_TENANT_BUDGET_EXCEEDED
  - PH1_TENANT_INTERNAL_PIPELINE_ERROR

### capability_id: TENANT_DECISION_COMPUTE
- input_schema:
  - bounded envelope (`correlation_id`, `turn_id`, `max_candidates`, `max_missing_fields`, `max_diagnostics`)
  - normalized policy posture (`identity_known`, `candidate_count`, optional selected tenant/policy refs, disabled/policy-block flags, `multiple_match`)
  - invariants (`deterministic=true`, `no_permission_decision=true`, `no_cross_tenant_access=true`)
- output_schema:
  - deterministic decision (`status`: `OK | NEEDS_CLARIFY | REFUSED | FAIL`)
  - optional tenant context (`tenant_id`, `policy_context_ref`, optional locale) when `status=OK`
  - bounded `missing_fields` list (single `tenant_choice` when `status=NEEDS_CLARIFY`)
  - reason code (`TENANT_NOT_FOUND | TENANT_MULTI_MATCH | TENANT_DISABLED | TENANT_POLICY_BLOCKED` and internal fallbacks)
  - invariants (`deterministic=true`, `no_permission_decision=true`, `no_cross_tenant_access=true`)
- allowed_callers: SELENE_OS_ONLY
- side_effects: NONE
- failure_modes: INPUT_SCHEMA_INVALID, TENANT_NOT_FOUND, TENANT_MULTI_MATCH, TENANT_DISABLED, TENANT_POLICY_BLOCKED, INTERNAL_PIPELINE_ERROR
- reason_codes:
  - PH1_TENANT_OK_DECISION_COMPUTE
  - TENANT_NOT_FOUND
  - TENANT_MULTI_MATCH
  - TENANT_DISABLED
  - TENANT_POLICY_BLOCKED
  - PH1_TENANT_INPUT_SCHEMA_INVALID
  - PH1_TENANT_INTERNAL_PIPELINE_ERROR

## Constraints
- Engines never call engines directly; Selene OS orchestrates all sequencing.
- PH1.TENANT never decides permissions and never changes gate ordering.
- Unknown identity without signed-in user context must not auto-select tenant.
- No cross-tenant access: outputs are scoped to one resolved tenant only.

## Related Engine Boundaries
- `PH1.VOICE.ID`: supplies identity assertions used by PH1.TENANT input context.
- `PH1.QUOTA`: consumes resolved tenant context as downstream scope input.
- `PH1.GOV`: consumes resolved tenant context for governance actions.
- `PH1.X`: handles clarify prompts for tenant choice; PH1.TENANT emits clarify posture only.
