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

## Gold-Case Capture Wiring (Round-2 Step 8)
- Selene OS now emits deterministic `GoldCaseCapture` envelopes for PH1.D provider-boundary outcomes through PH1.FEEDBACK wiring (`crates/selene_os/src/ph1feedback.rs`).
- PH1.D trigger set:
  - provider `ERROR`
  - provider validation `SCHEMA_FAIL`
  - low-confidence STT provider output
- Each PH1.D capture includes:
  - pending `gold_case_id`
  - bounded `reason_code_chain` (provider reason + low-confidence reason when applicable)
  - deterministic clustering keys (`primary_failure_fingerprint`, `secondary_failure_fingerprint`)
  - owner marker `PH1.D`
- PH1.D sourced captures are fail-closed validated and represented as PH1.FEEDBACK improvement-path events (no direct runtime authority).

## Builder Remediation Governance Lock (5H Step 12)
- PH1.D recurring unresolved fingerprints may flow to Builder proposal intake only through Selene OS mapping:
  - `map_recurring_failure_cluster_to_builder_offline_input(...)`
- Promotion remains fail-closed behind explicit human approval proofs:
  - `code_permission_gate_passed`
  - `launch_permission_gate_passed`
  - `release_hard_gate_passed`
- Builder authority boundary is unchanged:
  - Builder may propose remediation patches.
  - Builder cannot auto-ship code or launch without approval evidence.
