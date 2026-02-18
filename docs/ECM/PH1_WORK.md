# PH1_WORK ECM (Design vNext)

## Engine Header
- engine_id: PH1.WORK
- role: deterministic WorkOrder ledger decision engine (append-only + idempotency + tenant-scope enforcement)
- placement: ENTERPRISE_SUPPORT

## Capability List

### capability_id: WORK_POLICY_EVALUATE
- input_schema:
  - bounded envelope (`correlation_id`, `turn_id`, `max_payload_bytes`, `max_diagnostics`)
  - work-order scope (`tenant_id`, `work_order_id`, `event_type`, `created_at`)
  - event payload (`payload_min`)
  - idempotency posture (optional `idempotency_key`, `idempotency_required`, `idempotency_duplicate`)
  - persisted-state guard flags (`append_only_violation`, `tenant_scope_mismatch`)
  - invariants (`deterministic_replay_order=true`, `no_silent_conflict_merge=true`)
- output_schema:
  - deterministic policy posture (`event_valid`, `append_allowed`)
  - duplicate/violation flags (`idempotency_duplicate`, `append_only_violation`, `tenant_scope_mismatch`)
  - `payload_min_hash` + reason-coded output invariants
- allowed_callers: SELENE_OS_ONLY
- side_effects: NONE
- failure_modes: INPUT_SCHEMA_INVALID, EVENT_INVALID, APPEND_ONLY_VIOLATION, TENANT_MISMATCH, BUDGET_EXCEEDED, INTERNAL_PIPELINE_ERROR
- reason_codes:
  - PH1_WORK_OK_POLICY_EVALUATE
  - WORK_EVENT_INVALID
  - WORK_APPEND_ONLY_VIOLATION
  - WORK_IDEMPOTENCY_DUP
  - WORK_TENANT_MISMATCH
  - PH1_WORK_INPUT_SCHEMA_INVALID
  - PH1_WORK_BUDGET_EXCEEDED
  - PH1_WORK_INTERNAL_PIPELINE_ERROR

### capability_id: WORK_DECISION_COMPUTE
- input_schema:
  - bounded envelope (`correlation_id`, `turn_id`, `max_payload_bytes`, `max_diagnostics`)
  - work-order scope (`tenant_id`, `work_order_id`, `event_type`)
  - normalized policy posture (`event_valid`, `append_allowed`, duplicate/violation flags)
  - event-id selectors (`existing_event_id_on_duplicate`, `proposed_event_id`)
  - invariants (`deterministic_replay_order=true`, `no_silent_conflict_merge=true`)
- output_schema:
  - deterministic decision (`status=OK|REFUSED|FAIL`)
  - optional `work_order_event_id` (required for `OK`)
  - `idempotency_no_op` (duplicate path only)
  - reason code + invariants
- allowed_callers: SELENE_OS_ONLY
- side_effects: NONE
- failure_modes: INPUT_SCHEMA_INVALID, EVENT_INVALID, APPEND_ONLY_VIOLATION, TENANT_MISMATCH, INTERNAL_PIPELINE_ERROR
- reason_codes:
  - PH1_WORK_OK_DECISION_COMPUTE
  - WORK_EVENT_INVALID
  - WORK_APPEND_ONLY_VIOLATION
  - WORK_IDEMPOTENCY_DUP
  - WORK_TENANT_MISMATCH
  - PH1_WORK_INPUT_SCHEMA_INVALID
  - PH1_WORK_INTERNAL_PIPELINE_ERROR

## Constraints
- Engines never call engines directly; Selene OS orchestrates all sequencing.
- PH1.WORK is deterministic for identical input snapshots.
- PH1.WORK does not assign authority, execute side effects, or mutate non-work-order scope.
- Append-only and idempotency guarantees are fail-closed.

## Related Engine Boundaries
- `SELENE_OS_CORE_TABLES`: persists ledger/current records; PH1.WORK validates append posture only.
- `PH1.SCHED`: consumes deterministic work-order state and must not bypass PH1.WORK append discipline.
- `PH1.LEASE`: lease ownership constrains who can emit step events; PH1.WORK enforces deterministic event decisions.
- `PH1.EXPORT`: compliance exports depend on replay-safe ordering produced by this boundary.
- `PH1.J`: PH1.WORK decisions must be reason-coded for audit integrity.
