# PH1_MULTI ECM (Design vNext)

## Engine Header
- engine_id: PH1.MULTI
- role: Multimodal context fusion
- placement: TURN_OPTIONAL

## Capability List

### capability_id: MULTI_BUNDLE_COMPOSE
- input_schema:
  - bounded request envelope (`correlation_id`, `turn_id`, `max_signals`, `max_bundle_items`, `privacy_scope_required=true`)
  - bounded signal list from OS (`signal_id`, `source_engine`, `modality`, `hint_key`, `hint_value`, optional `evidence_ref`, confidence)
  - LISTEN-derived signals must come from Selene OS-validated `LISTEN_SIGNAL_FILTER=OK` outputs only
- output_schema:
  - one selected signal id
  - ordered bundle items (deterministic rank)
  - boundary flags: `evidence_backed=true`, `privacy_scoped=true`, `no_execution_authority=true`
- allowed_callers: SELENE_OS_ONLY
- side_effects: NONE
- failure_modes: INPUT_SCHEMA_INVALID, UPSTREAM_INPUT_MISSING, BUDGET_EXCEEDED, PRIVACY_SCOPE_REQUIRED, VALIDATION_FAILED, INTERNAL_PIPELINE_ERROR
- reason_codes:
  - PH1_MULTI_INPUT_SCHEMA_INVALID
  - PH1_MULTI_UPSTREAM_INPUT_MISSING
  - PH1_MULTI_BUDGET_EXCEEDED
  - PH1_MULTI_PRIVACY_SCOPE_REQUIRED
  - PH1_MULTI_VALIDATION_FAILED
  - PH1_MULTI_INTERNAL_PIPELINE_ERROR

### capability_id: MULTI_SIGNAL_ALIGN
- input_schema:
  - selected signal id
  - ordered bundle items from `MULTI_BUNDLE_COMPOSE`
  - deterministic envelope constraints
- output_schema:
  - validation_result (`OK|FAIL`)
  - bounded diagnostics
  - boundary flags: `evidence_backed=true`, `privacy_scoped=true`, `no_execution_authority=true`
- allowed_callers: SELENE_OS_ONLY
- side_effects: NONE
- failure_modes: VALIDATION_FAILED, INPUT_SCHEMA_INVALID, BUDGET_EXCEEDED
- reason_codes:
  - PH1_MULTI_OK_SIGNAL_ALIGN
  - PH1_MULTI_VALIDATION_FAILED
  - PH1_MULTI_INPUT_SCHEMA_INVALID
  - PH1_MULTI_BUDGET_EXCEEDED

## Constraints
- Engines never call engines directly; Selene OS orchestrates all sequencing.
- Non-authoritative assist engine; outputs are advisory only.
- No execution path and no authority mutation.
- Vision/document modality entries require evidence refs; otherwise contract fails closed.
- LISTEN-derived environment hints are advisory-only and must never mutate meaning.
- PH1.MULTI output is context input only after Selene OS verifies `MULTI_SIGNAL_ALIGN=OK`; PH1.CONTEXT still applies `CONTEXT_BUNDLE_TRIM` fail-closed checks.
