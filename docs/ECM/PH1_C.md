# PH1.C ECM Spec

## Engine Header
- `engine_id`: `PH1.C`
- `purpose`: Persist deterministic STT transcript gate outcomes (`transcript_ok` / `transcript_reject`) through conversation/audit ledgers.
- `data_owned`: `conversation_ledger` writes in PH1.C scope, `audit_events` writes in PH1.C scope
- `version`: `v1`
- `status`: `ACTIVE`

## Capability List

### `PH1C_TRANSCRIPT_OK_COMMIT_ROW`
- `name`: Commit accepted transcript turn plus audit events
- `input_schema`: `(now, tenant_id, correlation_id, turn_id, session_id?, user_id, device_id, transcript_text, transcript_hash, language_tag, confidence_bucket, route_class_used, attempt_count, candidate_count, selected_slot, mode_used, second_pass_used, critical_spans?, idempotency_key)`
- `output_schema`: `Result<Ph1cTranscriptOkCommitResult, StorageError>`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `DECLARED (DB_WRITE)`

### `PH1C_TRANSCRIPT_REJECT_COMMIT_ROW`
- `name`: Commit transcript rejection audit events
- `input_schema`: `(now, tenant_id, correlation_id, turn_id, session_id?, user_id, device_id, reject_reason_code, retry_advice, transcript_hash?, route_class_used, attempt_count, candidate_count, selected_slot, mode_used, second_pass_used, uncertain_spans?, idempotency_key)`
- `output_schema`: `Result<Ph1cTranscriptRejectCommitResult, StorageError>`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `DECLARED (DB_WRITE)`

### `PH1C_READ_VOICE_TRANSCRIPT_ROWS`
- `name`: Read voice transcript ledger rows for one correlation thread
- `input_schema`: `correlation_id`
- `output_schema`: `ConversationTurnRecord[]`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `NONE`

## Failure Modes + Reason Codes
- reject/fail-closed reason codes include:
  - `STT_FAIL_EMPTY`
  - `STT_FAIL_LOW_CONFIDENCE`
  - `STT_FAIL_LOW_COVERAGE`
  - `STT_FAIL_LANGUAGE_MISMATCH`
  - `STT_FAIL_AUDIO_DEGRADED`
  - `STT_FAIL_BUDGET_EXCEEDED`
- scope/contract failures are fail-closed with deterministic PH1.C reason coding.

## Audit Emission Requirements Per Capability
- `PH1C_TRANSCRIPT_OK_COMMIT_ROW` must emit PH1.J audit rows including:
  - `TranscriptOk`
  - `SttCandidateEval`
- `PH1C_TRANSCRIPT_REJECT_COMMIT_ROW` must emit PH1.J audit rows including:
  - `TranscriptReject`
  - `SttCandidateEval`
- `payload_min` remains bounded and provider-invisible while preserving arbitration indicators (`route_class_used`, `attempt_count`, `candidate_count`, `selected_slot`, `mode_used`, `second_pass_used`).
- `evidence_ref` (when present) must carry bounded transcript span references only.

## Sources
- `crates/selene_storage/src/repo.rs` (`Ph1cSttRepo`)
- `docs/DB_WIRING/PH1_C.md`

## Related Engine Boundary (`PH1.ENDPOINT`)
- PH1.C may consume one Selene OS-curated endpoint hint from PH1.ENDPOINT before transcript finalization.
- PH1.C must not treat PH1.ENDPOINT output as transcript authority; PH1.C remains the transcript gate owner.

## Related Engine Boundary (`PH1.KNOW`)
- PH1.C may consume one Selene OS-curated PH1.KNOW vocabulary hint bundle before transcript finalization.
- PH1.C must not treat PH1.KNOW output as transcript authority; PH1.C remains the transcript gate owner.
- PH1.KNOW-derived hints must remain tenant-scoped and authorized-only in PH1.C capability execution.

## Related Engine Boundary (`PH1.QUOTA`)
- PH1.C capability execution may be pre-gated by Selene OS using `PH1.QUOTA` lane decisions.
- If quota posture is `REFUSE`, PH1.C capability calls must not run.
- If quota posture is `WAIT`, Selene OS may pause before PH1.C; PH1.C output contracts remain unchanged when resumed.
