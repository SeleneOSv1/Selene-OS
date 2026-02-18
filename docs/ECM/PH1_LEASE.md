# PH1_LEASE ECM (Design vNext)

## Engine Header
- engine_id: PH1.LEASE
- role: deterministic lease ownership decision engine for WorkOrder executor locking
- placement: ENTERPRISE_SUPPORT

## Capability List

### capability_id: LEASE_POLICY_EVALUATE
- input_schema:
  - bounded envelope (`correlation_id`, `turn_id`, `max_diagnostics`, `max_ttl_ms`)
  - WorkOrder scope (`tenant_id`, `work_order_id`)
  - lease command tuple (`lease_owner_id`, `operation`, optional `lease_token`, `requested_ttl_ms`, `now_ns`)
  - active lease snapshot (`active_lease_owner_id`, `active_lease_token`, `active_lease_expires_at_ns`)
  - invariants (`deterministic_takeover_from_ledger=true`, `one_active_lease_per_work_order=true`, `token_owner_required=true`)
- output_schema:
  - deterministic lease posture (`lease_exists`, `lease_expired`, `owner_match`, `token_match`, `ttl_in_bounds`, `grant_eligible`)
  - reason-coded policy result preserving lease invariants
- allowed_callers: SELENE_OS_ONLY
- side_effects: NONE
- failure_modes: INPUT_SCHEMA_INVALID, UPSTREAM_INPUT_MISSING, LEASE_TTL_OUT_OF_BOUNDS, LEASE_NOT_FOUND, LEASE_HELD_BY_OTHER, LEASE_TOKEN_INVALID, BUDGET_EXCEEDED, INTERNAL_PIPELINE_ERROR
- reason_codes:
  - PH1_LEASE_OK_POLICY_EVALUATE
  - LEASE_HELD_BY_OTHER
  - LEASE_TOKEN_INVALID
  - LEASE_TTL_OUT_OF_BOUNDS
  - LEASE_NOT_FOUND
  - PH1_LEASE_INPUT_SCHEMA_INVALID
  - PH1_LEASE_UPSTREAM_INPUT_MISSING
  - PH1_LEASE_BUDGET_EXCEEDED
  - PH1_LEASE_INTERNAL_PIPELINE_ERROR

### capability_id: LEASE_DECISION_COMPUTE
- input_schema:
  - bounded envelope (`correlation_id`, `turn_id`, `max_diagnostics`, `max_ttl_ms`)
  - WorkOrder scope (`tenant_id`, `work_order_id`)
  - normalized lease posture from evaluate step
  - lease decision context (operation, owner/token tuple, optional proposed lease token)
  - active lease snapshot metadata for bounded deny details
  - invariants (`deterministic_takeover_from_ledger=true`, `one_active_lease_per_work_order=true`, `token_owner_required=true`)
- output_schema:
  - deterministic decision (`LeaseGranted | LeaseDenied`)
  - active grant path (`lease_token`, `lease_expires_at_ns`) for acquire/renew
  - inactive grant path for release (token cleared)
  - bounded deny holder metadata (`held_by_owner_id`, `held_until_ns`) when applicable
  - `resume_from_ledger_required` flag for expired takeover path
- allowed_callers: SELENE_OS_ONLY
- side_effects: NONE
- failure_modes: INPUT_SCHEMA_INVALID, LEASE_TTL_OUT_OF_BOUNDS, LEASE_NOT_FOUND, LEASE_HELD_BY_OTHER, LEASE_TOKEN_INVALID, BUDGET_EXCEEDED, INTERNAL_PIPELINE_ERROR
- reason_codes:
  - PH1_LEASE_OK_DECISION_COMPUTE
  - LEASE_HELD_BY_OTHER
  - LEASE_TOKEN_INVALID
  - LEASE_TTL_OUT_OF_BOUNDS
  - LEASE_NOT_FOUND
  - PH1_LEASE_INPUT_SCHEMA_INVALID
  - PH1_LEASE_BUDGET_EXCEEDED
  - PH1_LEASE_INTERNAL_PIPELINE_ERROR

## Constraints
- Engines never call engines directly; Selene OS orchestrates all sequencing.
- Lease decisions are deterministic for identical input snapshots.
- PH1.LEASE must never grant authority or execute side effects.
- Renew/release paths are token-owner gated and fail closed on mismatch.
- Expired takeover requires deterministic resume from persisted ledger state only.

## Related Engine Boundaries
- `SELENE_OS_CORE_TABLES`: stores canonical lease state in `work_order_leases`; PH1.LEASE is decision-only.
- `PH1.WORK`: consumes lease posture before step event advancement.
- `PH1.SCHED`: retry/wait scheduling is separate and cannot bypass lease ownership checks.
- `PH1.EXPORT`: exports may include lease traces but PH1.LEASE does not generate export artifacts.
- `PH1.J`: lease decisions must emit reason-coded audit events.
