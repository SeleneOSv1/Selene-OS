# PH1_EXPORT ECM (Design vNext)

## Engine Header
- engine_id: PH1.EXPORT
- role: Compliance export proof builder with deterministic redaction and tamper-evident hash output
- placement: ENTERPRISE_SUPPORT

## Capability List

### capability_id: EXPORT_ACCESS_EVALUATE
- input_schema:
  - bounded envelope (`correlation_id`, `turn_id`, `max_include_items`, `max_diagnostics`, `max_time_range_ms`)
  - requester + tenant context (`tenant_id`, `requester_user_id`)
  - export scope (`work_order_id` or `time_range`)
  - include list (`audit_events | work_order_ledger | conversation_turns`)
  - redaction policy reference + hard flags (`require_audit_event=true`, `disallow_raw_audio=true`)
- output_schema:
  - deterministic `export_scope_ref`
  - normalized include list + redaction policy echo
  - explicit guard flags (`requester_authorized`, `deterministic_redaction_required`, `raw_audio_excluded`, `audit_event_required`)
- allowed_callers: SELENE_OS_ONLY
- side_effects: NONE
- failure_modes: INPUT_SCHEMA_INVALID, UPSTREAM_INPUT_MISSING, EXPORT_NOT_AUTHORIZED, EXPORT_RANGE_TOO_LARGE, EXPORT_REDACTION_REQUIRED, EXPORT_FAILED, INTERNAL_PIPELINE_ERROR
- reason_codes:
  - PH1_EXPORT_OK_ACCESS_EVALUATE
  - PH1_EXPORT_INPUT_SCHEMA_INVALID
  - PH1_EXPORT_UPSTREAM_INPUT_MISSING
  - EXPORT_NOT_AUTHORIZED
  - EXPORT_RANGE_TOO_LARGE
  - EXPORT_REDACTION_REQUIRED
  - EXPORT_FAILED
  - PH1_EXPORT_INTERNAL_PIPELINE_ERROR

### capability_id: EXPORT_ARTIFACT_BUILD
- input_schema:
  - bounded envelope (`correlation_id`, `turn_id`, `max_include_items`, `max_diagnostics`, `max_time_range_ms`)
  - evaluated export context (`tenant_id`, `export_scope_ref`, `requester_user_id`)
  - include list + redaction policy
  - hard flags (`deterministic_redaction_required=true`, `raw_audio_excluded=true`, `audit_event_required=true`)
- output_schema:
  - `status=OK`
  - deterministic `export_artifact_id`, `export_hash` (64-hex), and `export_payload_ref`
  - bounded compliance flags (`redaction_applied`, `raw_audio_excluded`, `audit_event_emitted`, `deterministic_hash`)
- allowed_callers: SELENE_OS_ONLY
- side_effects: NONE
- failure_modes: INPUT_SCHEMA_INVALID, EXPORT_REDACTION_REQUIRED, EXPORT_FAILED, INTERNAL_PIPELINE_ERROR
- reason_codes:
  - PH1_EXPORT_OK_ARTIFACT_BUILD
  - PH1_EXPORT_INPUT_SCHEMA_INVALID
  - EXPORT_REDACTION_REQUIRED
  - EXPORT_FAILED
  - PH1_EXPORT_INTERNAL_PIPELINE_ERROR

## Constraints
- Engines never call engines directly; Selene OS orchestrates all sequencing.
- Must never export raw audio by default.
- Must never return success output unless audit emission is explicitly true.
- Tamper-evident hash output must be deterministic for identical input snapshots.
- Export logic is read-only/derivation-only in this runtime slice and cannot mutate authority/state.

## Related Engine Boundaries
- PH1.J boundary: export success must be represented by reason-coded audit proof events.
- PH1.WORK boundary: work-order ledger truth remains external; PH1.EXPORT only references scoped data.
- PH1.KMS boundary: no raw secret material is allowed in export payload/output fields.
