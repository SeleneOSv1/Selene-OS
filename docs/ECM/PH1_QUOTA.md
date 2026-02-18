# PH1_QUOTA ECM (Design vNext)

## Engine Header
- engine_id: PH1.QUOTA
- role: deterministic quota and budget decision engine for runtime lanes
- placement: ENTERPRISE_SUPPORT

## Capability List

### capability_id: QUOTA_POLICY_EVALUATE
- input_schema:
  - bounded envelope (`correlation_id`, `turn_id`, `max_diagnostics`, `max_wait_ms`)
  - scope metadata (`tenant_id`, optional `user_id`, optional `device_id`)
  - lane metadata (`operation_kind`, `capability_id`/`tool_name`)
  - timing/budget inputs (`now`, optional `cost_hint`)
  - deterministic guard flags (`rate_limit_exceeded`, `budget_exceeded`, `policy_blocked`, `wait_permitted`)
  - invariants (`deterministic=true`, `no_authority_grant=true`, `no_gate_order_change=true`)
- output_schema:
  - deterministic throttle posture (`throttle_cause`)
  - deterministic lane policy result (`allow_eligible`, `wait_permitted`, optional `wait_ms`, `refuse_required`)
  - reason-coded policy output + invariants
- allowed_callers: SELENE_OS_ONLY
- side_effects: NONE
- failure_modes: INPUT_SCHEMA_INVALID, UPSTREAM_INPUT_MISSING, QUOTA_RATE_LIMIT, QUOTA_BUDGET_EXCEEDED, QUOTA_POLICY_BLOCKED, BUDGET_EXCEEDED, INTERNAL_PIPELINE_ERROR
- reason_codes:
  - PH1_QUOTA_OK_POLICY_EVALUATE
  - QUOTA_RATE_LIMIT
  - QUOTA_BUDGET_EXCEEDED
  - QUOTA_POLICY_BLOCKED
  - PH1_QUOTA_INPUT_SCHEMA_INVALID
  - PH1_QUOTA_UPSTREAM_INPUT_MISSING
  - PH1_QUOTA_BUDGET_EXCEEDED
  - PH1_QUOTA_INTERNAL_PIPELINE_ERROR

### capability_id: QUOTA_DECISION_COMPUTE
- input_schema:
  - bounded envelope (`correlation_id`, `turn_id`, `max_diagnostics`, `max_wait_ms`)
  - scope metadata (`tenant_id`, `operation_kind`)
  - normalized policy posture (`throttle_cause`, `allow_eligible`, `wait_permitted`, optional `wait_ms`, `refuse_required`)
  - invariants (`deterministic=true`, `no_authority_grant=true`, `no_gate_order_change=true`)
- output_schema:
  - deterministic decision (`ALLOW | WAIT | REFUSE`)
  - optional `wait_ms` only for `WAIT`
  - reason code + invariants (`no_authority_grant=true`, `no_gate_order_change=true`)
- allowed_callers: SELENE_OS_ONLY
- side_effects: NONE
- failure_modes: INPUT_SCHEMA_INVALID, QUOTA_RATE_LIMIT, QUOTA_BUDGET_EXCEEDED, QUOTA_POLICY_BLOCKED, INTERNAL_PIPELINE_ERROR
- reason_codes:
  - PH1_QUOTA_OK_DECISION_COMPUTE
  - QUOTA_RATE_LIMIT
  - QUOTA_BUDGET_EXCEEDED
  - QUOTA_POLICY_BLOCKED
  - PH1_QUOTA_INPUT_SCHEMA_INVALID
  - PH1_QUOTA_INTERNAL_PIPELINE_ERROR

## Constraints
- Engines never call engines directly; Selene OS orchestrates all sequencing.
- PH1.QUOTA is deterministic for identical input snapshots.
- PH1.QUOTA never grants authority and never changes gate ordering.
- PH1.QUOTA only emits lane posture; it does not execute operations.

## Related Engine Boundaries
- `PH1.TENANT`: provides tenant-scope prerequisite context for quota checks.
- `PH1.COST`: advisory cost hints may feed quota inputs, but PH1.QUOTA remains authoritative for lane decision.
- `PH1.C` / `PH1.TTS` / `PH1.E` / `PH1.SCHED` / `PH1.EXPORT`: consume quota posture only through Selene OS, never direct engine-to-engine calls.
- `PH1.J`: quota decisions must be auditable with reason codes.
