# PH1.D ECM Spec

## Engine Header
- `engine_id`: `PH1.D`
- `purpose`: Persist deterministic PH1.D model-boundary outputs (`chat`, `intent`, `clarify`, `analysis`, `fail_closed`) as bounded audit rows.
- `data_owned`: `audit_events` writes in PH1.D scope
- `version`: `v1`
- `status`: `ACTIVE`

## Capability List

### `PH1D_CHAT_COMMIT_ROW`
- `name`: Commit PH1.D chat output decision
- `input_schema`: `(now, tenant_id, correlation_id, turn_id, session_id?, user_id, device_id, request_id, prompt_template_version, output_schema_hash, tool_catalog_hash, policy_context_hash, transcript_hash, model_id, model_route_class, temperature_bp, max_tokens, reason_code, idempotency_key)`
- `output_schema`: `Result<AuditEventId, StorageError>`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `DECLARED (DB_WRITE)`

### `PH1D_INTENT_COMMIT_ROW`
- `name`: Commit PH1.D intent refinement decision
- `input_schema`: `(now, tenant_id, correlation_id, turn_id, session_id?, user_id, device_id, request_id, prompt_template_version, output_schema_hash, tool_catalog_hash, policy_context_hash, transcript_hash, model_id, model_route_class, temperature_bp, max_tokens, refined_intent_type, reason_code, idempotency_key)`
- `output_schema`: `Result<AuditEventId, StorageError>`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `DECLARED (DB_WRITE)`

### `PH1D_CLARIFY_COMMIT_ROW`
- `name`: Commit PH1.D clarify decision
- `input_schema`: `(now, tenant_id, correlation_id, turn_id, session_id?, user_id, device_id, request_id, prompt_template_version, output_schema_hash, tool_catalog_hash, policy_context_hash, transcript_hash, model_id, model_route_class, temperature_bp, max_tokens, what_is_missing, reason_code, idempotency_key)`
- `output_schema`: `Result<AuditEventId, StorageError>`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `DECLARED (DB_WRITE)`

### `PH1D_ANALYSIS_COMMIT_ROW`
- `name`: Commit PH1.D analysis decision
- `input_schema`: `(now, tenant_id, correlation_id, turn_id, session_id?, user_id, device_id, request_id, prompt_template_version, output_schema_hash, tool_catalog_hash, policy_context_hash, transcript_hash, model_id, model_route_class, temperature_bp, max_tokens, analysis_kind, reason_code, idempotency_key)`
- `output_schema`: `Result<AuditEventId, StorageError>`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `DECLARED (DB_WRITE)`

### `PH1D_FAIL_CLOSED_COMMIT_ROW`
- `name`: Commit PH1.D fail-closed decision
- `input_schema`: `(now, tenant_id, correlation_id, turn_id, session_id?, user_id, device_id, request_id, prompt_template_version, output_schema_hash, tool_catalog_hash, policy_context_hash, transcript_hash, model_id?, model_route_class?, temperature_bp?, max_tokens?, fail_code, reason_code, idempotency_key)`
- `output_schema`: `Result<AuditEventId, StorageError>`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `DECLARED (DB_WRITE)`

### `PH1D_READ_AUDIT_ROWS`
- `name`: Read PH1.D audit rows for one correlation thread
- `input_schema`: `correlation_id`
- `output_schema`: `AuditEvent[]`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `NONE`

## Failure Modes + Reason Codes
- PH1.D deterministic failure reason codes:
  - `D_FAIL_INVALID_SCHEMA`
  - `D_FAIL_FORBIDDEN_OUTPUT`
  - `D_FAIL_SAFETY_BLOCK`
  - `D_FAIL_TIMEOUT`
  - `D_FAIL_BUDGET_EXCEEDED`
- every non-fail output is still reason-coded; scope/contract violations fail closed.

## Audit Emission Requirements Per Capability
- write capabilities emit PH1.J rows with bounded keys only:
  - `decision`
  - `output_mode`
  - `refined_intent_type`
  - `what_is_missing`
  - `analysis_kind`
  - `fail_code`
  - `request_id`
  - `prompt_template_version`
  - `output_schema_hash`
  - `tool_catalog_hash`
  - `policy_context_hash`
  - `transcript_hash`
  - `model_id`
  - `model_route_class`
  - `temperature_bp`
  - `max_tokens`
- read capability emits audit only in explicit replay/diagnostic mode.

## Sources
- `crates/selene_storage/src/repo.rs` (`Ph1dRouterRepo`)
- `docs/DB_WIRING/PH1_D.md`

## Related Engine Boundary (Provider Gateway Lock)
- `PH1.D` is the only runtime provider/model boundary for LLM and OCR service calls.
- Upstream analyzers (`PH1.VISION`, `PH1.DOC`) and deterministic parser owner (`PH1.NLP`) may request assistance only via Selene OS routing into PH1.D.
- PH1.D provider output is advisory until schema/policy validation succeeds and Selene OS accepts it under normal gate order.
- Provider-assisted outputs cannot bypass clarify owner, access gates, simulation gates, or audit/idempotency requirements.

## Provider Adapter Runtime Contract (Step-1 Lock)

### `PH1D_PROVIDER_CALL_EXECUTE`
- `name`: Execute one provider call under PH1.D boundary.
- `input_schema`:
  - `schema_version`, `correlation_id`, `turn_id`, `tenant_id`, `request_id`, `idempotency_key`
  - `provider_task` (`LLM_INTERPRET | OCR_TEXT_EXTRACT`)
  - `provider_route_class` (`PRIMARY | SECONDARY | TERTIARY`)
  - `provider_id`, `model_id`
  - `timeout_ms`, `retry_budget`, `temperature_bp`, `max_tokens`
  - `prompt_template_version`, `output_schema_hash`, `tool_catalog_hash`, `policy_context_hash`, `transcript_hash?`
  - `input_payload_ref`, `input_payload_kind`, `input_payload_hash`, `input_payload_inline?`, `input_mime_type?`
  - `safety_tier`, `privacy_mode`, `do_not_disturb`
- `output_schema`:
  - `schema_version`, `correlation_id`, `turn_id`, `request_id`, `idempotency_key`
  - `provider_call_id`, `provider_id`, `provider_task`, `provider_model`
  - `provider_status` (`OK | TIMEOUT | BUDGET_EXCEEDED | SAFETY_BLOCK | RATE_LIMIT | PROVIDER_ERROR`)
  - `provider_latency_ms`, `provider_cost_microunits`, `provider_confidence_bp?`
  - `raw_output_hash`, `normalized_output_json`
  - `validation_status` (`SCHEMA_OK | SCHEMA_FAIL | POLICY_FAIL`)
  - `reason_code`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `DECLARED (PROVIDER_CALL + AUDIT_WRITE)`

### `PH1D_PROVIDER_OUTPUT_VALIDATE`
- `name`: Validate and normalize provider output before mode decision emission.
- `input_schema`:
  - provider response envelope from `PH1D_PROVIDER_CALL_EXECUTE`
  - PH1.D expected output mode contract (`chat|intent|clarify|analysis|fail_closed`)
- `output_schema`:
  - `validation_status`
  - `normalized_mode_output` (bounded)
  - `reason_code`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `NONE`

### Runtime Checklist Gate (must pass before provider path is considered active)
1. Kernel contract types + validators land.
2. Provider adapter trait + OpenAI implementation land.
3. PH1.D route policy (`LLM_INTERPRET`/`OCR_TEXT_EXTRACT`) lands.
4. Fail-closed mapping for timeout/rate-limit/safety/schema drift lands.
5. Audit payload contains bounded provider metadata keys.
6. OS wiring refuses malformed provider envelopes deterministically.
7. Test suite proves both happy-path and fail-closed behavior.
