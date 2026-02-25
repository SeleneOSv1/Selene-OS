# PH1.TTS ECM Spec

## Engine Header
- `engine_id`: `PH1.TTS`
- `purpose`: Persist deterministic TTS rendering/playback outcomes (`render_summary`, `started`, `canceled`, `failed`) as bounded audit rows.
- `data_owned`: `audit_events` writes in PH1.TTS scope
- `version`: `v1`
- `status`: `ACTIVE`
- `related_inputs`: Optional `pron_pack_ref` hints from `PH1.PRON` (tenant-scoped; user-scoped only with explicit consent)

## Capability List

### `PH1TTS_RENDER_SUMMARY_COMMIT_ROW`
- `name`: Commit TTS render route/mode summary
- `input_schema`: `(now, tenant_id, correlation_id, turn_id, session_id?, user_id, device_id, route_class_used, mode_used, voice_id, reason_code, idempotency_key)`
- `output_schema`: `Result<AuditEventId, StorageError>`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `DECLARED (DB_WRITE)`

### `PH1TTS_STARTED_COMMIT_ROW`
- `name`: Commit TTS started marker
- `input_schema`: `(now, tenant_id, correlation_id, turn_id, session_id?, user_id, device_id, answer_id, reason_code, idempotency_key)`
- `output_schema`: `Result<AuditEventId, StorageError>`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `DECLARED (DB_WRITE)`

### `PH1TTS_CANCELED_COMMIT_ROW`
- `name`: Commit TTS canceled marker
- `input_schema`: `(now, tenant_id, correlation_id, turn_id, session_id?, user_id, device_id, answer_id, stop_reason, reason_code, idempotency_key)`
- `output_schema`: `Result<AuditEventId, StorageError>`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `DECLARED (DB_WRITE)`

### `PH1TTS_FAILED_COMMIT_ROW`
- `name`: Commit TTS failed marker
- `input_schema`: `(now, tenant_id, correlation_id, turn_id, session_id?, user_id, device_id, answer_id, fail_code, reason_code, idempotency_key)`
- `output_schema`: `Result<AuditEventId, StorageError>`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `DECLARED (DB_WRITE)`

### `PH1TTS_READ_AUDIT_ROWS`
- `name`: Read PH1.TTS audit rows for one correlation thread
- `input_schema`: `correlation_id`
- `output_schema`: `AuditEvent[]`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `NONE`

## Failure Modes + Reason Codes
- PH1.TTS deterministic fail reason codes include:
  - `TTS_FAIL_POLICY_RESTRICTED`
  - `TTS_FAIL_NETWORK_UNAVAILABLE`
  - `TTS_FAIL_PROVIDER_TIMEOUT`
  - `TTS_FAIL_QUOTA_THROTTLED`
- storage scope/idempotency failures are fail-closed and reason-coded.

## Audit Emission Requirements Per Capability
- write capabilities emit PH1.J rows with bounded payload keys:
  - `route_class_used`, `mode_used`, `voice_id`
  - `answer_id`, `stop_reason`, `fail_code`
- `PH1TTS_READ_AUDIT_ROWS` emits audit only in replay/diagnostic mode.

## Sources
- `crates/selene_storage/src/repo.rs` (`Ph1TtsRepo`)
- `docs/DB_WIRING/PH1_TTS.md`

## Related Engine Boundary (`PH1.KNOW`)
- PH1.TTS may consume Selene OS-curated PH1.KNOW pronunciation hints as optional render assist inputs.
- PH1.TTS capability execution remains deterministic rendering-only and must not treat PH1.KNOW hints as semantic authority.
- PH1.KNOW-derived hints must remain tenant-scoped and authorized-only in PH1.TTS capability execution.

## Related Engine Boundary (`PH1.EMO.GUIDE`)
- PH1.TTS may consume Selene OS-curated PH1.EMO.GUIDE style-profile hints only when EMO.GUIDE validation is `OK`.
- PH1.TTS must treat PH1.EMO.GUIDE output as advisory tone policy only; it cannot modify factual meaning, intent outcomes, or execution ordering.

## Related Engine Boundary (`PH1.PERSONA`)
- PH1.TTS may consume Selene OS-curated PH1.PERSONA style/delivery profile hints only when `PERSONA_PROFILE_VALIDATE` is `OK`.
- PH1.TTS must treat PH1.PERSONA output as advisory rendering policy only; it cannot modify factual meaning, intent outcomes, confirmation semantics, or execution ordering.

## FDX Design Lock (Section 5F)
- PH1.TTS owns chunked speech playback and immediate cancel responsiveness in duplex sessions.
- PH1.TTS must support interrupt-safe cancellation from PH1.X with deterministic reason-coded audit rows.
- PH1.TTS must preserve bounded cancel latency expectations (`barge-in detect -> TTS cancel` p95 <= 120ms end-to-end path).
- PH1.TTS remains rendering-only authority and must not perform interruption policy decisions.
- PH1.TTS must maintain replay-safe resume snapshot compatibility with PH1.X continuity policies.
