# PH1_HEALTH ECM (Design v1, Display-Only)

## Engine Header
- engine_id: PH1.HEALTH
- role: health issue/report projection for app display
- placement: ENTERPRISE_SUPPORT
- allowed_callers: SELENE_OS_ONLY (including app ingress adapters)

## Capability List

### capability_id: HEALTH_SNAPSHOT_READ
- input_schema:
  - `tenant_id`
  - `viewer_user_id`
  - optional filters (`open_only`, `severity`, `engine_owner`, `escalated_only`)
- output_schema:
  - summary counters (`open`, `critical`, `resolved_24h`, `escalated_24h`, `mttr`)
  - bounded issue list rows
- side_effects: NONE
- failure_modes: INPUT_SCHEMA_INVALID, TENANT_SCOPE_INVALID
- reason_codes:
  - PH1_HEALTH_OK_SNAPSHOT_READ
  - PH1_HEALTH_INPUT_SCHEMA_INVALID
  - PH1_HEALTH_TENANT_SCOPE_INVALID

### capability_id: HEALTH_ISSUE_TIMELINE_READ
- input_schema:
  - `tenant_id`
  - `viewer_user_id`
  - `issue_id`
- output_schema:
  - issue metadata
  - ordered action timeline (`attempt_no`, `action_id`, `result`, `reason_code`, `started_at`)
  - escalation refs (`bcast_id`, `ack_state`) when present
- side_effects: NONE
- failure_modes: INPUT_SCHEMA_INVALID, ISSUE_NOT_FOUND
- reason_codes:
  - PH1_HEALTH_OK_ISSUE_TIMELINE_READ
  - PH1_HEALTH_INPUT_SCHEMA_INVALID
  - PH1_HEALTH_ISSUE_NOT_FOUND

### capability_id: HEALTH_UNRESOLVED_SUMMARY_READ
- input_schema:
  - `tenant_id`
  - `viewer_user_id`
  - optional `breach_only`
- output_schema:
  - unresolved issue summary
  - SLA breach indicators
  - escalation-state indicators
- side_effects: NONE
- failure_modes: INPUT_SCHEMA_INVALID
- reason_codes:
  - PH1_HEALTH_OK_UNRESOLVED_SUMMARY_READ
  - PH1_HEALTH_INPUT_SCHEMA_INVALID

## Hard Rules
- Display-only engine in v1.
- No remediation execution.
- No simulation calls.
- No authority mutation.
- No engine-to-engine direct action calls.

## Desktop UI Contract
- Health screen must use ChatGPT-style shell layout:
  - left sidebar,
  - center content list,
  - right detail panel.
- PH1.HEALTH supplies content/state only; shell layout contract is fixed by app UI.

## Implementation References
- kernel contracts: `crates/selene_kernel_contracts/src/ph1health.rs`
- engine runtime: `crates/selene_engines/src/ph1health.rs`
- os wiring: `crates/selene_os/src/ph1health.rs`
