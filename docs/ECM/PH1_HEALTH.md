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

### capability_id: HEALTH_REPORT_QUERY_READ
- input_schema:
  - `tenant_id`
  - `viewer_user_id`
  - `report_kind`
  - `time_range` (`from_utc`, `to_utc`)
  - optional `engine_owner_filter`
  - `company_scope` (`TENANT_ONLY | CROSS_TENANT_TENANT_ROWS`)
  - optional `company_ids[]`
  - optional `country_codes[]`
  - optional `escalated_only`
  - optional `unresolved_only`
  - optional `display_target` (`DESKTOP | PHONE`)
  - `page_action` (`FIRST | NEXT | PREV | REFRESH`)
  - optional `page_cursor`
  - optional `report_context_id`
- output_schema:
  - `report_context_id`
  - `report_revision`
  - `normalized_query`
  - tenant-row report rows with unresolved/escalation proof fields
  - paging object (`has_next`, `has_prev`, `next_cursor`, `prev_cursor`)
  - `display_target_applied`
  - optional `requires_clarification`
- side_effects: NONE
- failure_modes:
  - INPUT_SCHEMA_INVALID
  - DISPLAY_TARGET_REQUIRED
  - DATE_RANGE_INVALID
  - COUNTRY_FILTER_INVALID
  - CROSS_TENANT_UNAUTHORIZED
  - REPORT_CONTEXT_NOT_FOUND
  - PAGE_CURSOR_INVALID
- reason_codes:
  - PH1_HEALTH_OK_REPORT_QUERY_READ
  - PH1_HEALTH_DISPLAY_TARGET_REQUIRED
  - PH1_HEALTH_DATE_RANGE_INVALID
  - PH1_HEALTH_COUNTRY_FILTER_INVALID
  - PH1_HEALTH_CROSS_TENANT_UNAUTHORIZED
  - PH1_HEALTH_REPORT_CONTEXT_NOT_FOUND
  - PH1_HEALTH_PAGE_CURSOR_INVALID

## Resolution-Proof and Escalation Payload Requirements
- Resolved status must be evidence-backed:
  - `issue_fingerprint`
  - `verification_window` metadata
  - `recurrence_observed` boolean
  - recurrence evidence refs when recurrence is present
- Escalated/unresolved rows must expose:
  - `impact_summary`
  - `attempted_fix_actions[]`
  - `current_monitoring_evidence`
  - `unresolved_reason_exact`
  - `bcast_id`/`ack_state` when escalated

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
