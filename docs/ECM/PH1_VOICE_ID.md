# PH1.VOICE.ID ECM Spec

## Engine Header
- `engine_id`: `PH1.VOICE.ID`
- `implementation_id`: `PH1.VOICE.ID.001`
- `active_implementation_ids`: `[PH1.VOICE.ID.001]`
- `purpose`: Persist voice enrollment session/sample/profile lifecycle under deterministic idempotency and append-only sample history.
- `data_owned`: `voice_enrollment_sessions`, `voice_enrollment_samples`, `voice_profiles`
- `version`: `v1`
- `status`: `ACTIVE`
- `related_inputs`: Optional pronunciation-hint packs from `PH1.PRON` for robustness only (no identity authority changes)

## Phone-First Contract Lock
- Voice-ID runtime is phone-first; local artifacts are runtime source-of-speed, cloud artifacts are continuity source-of-truth.
- Local artifact set (`ACTIVE + N-1`) must be maintained and synced continuously via Engine B outbox handoff.
- Raw audio sync is forbidden by default; only bounded references/features/manifests are permitted unless explicit consent policy allows otherwise.

## Capability List

### `VID_ASSERT_SPEAKER_DECISION`
- `name`: Deterministic speaker assertion (OK or UNKNOWN)
- `input_schema`: `(processed_audio_stream_ref, vad_events[], device_id, device_trust_level, device_owner_user_id?, session_state_ref, wake_event?, tts_playback_active, risk_signals[])`
- `output_schema`: `SpeakerAssertionOk | SpeakerAssertionUnknown`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `NONE`

### `VID_ENROLL_START_DRAFT_ROW`
- `name`: Start enrollment session draft
- `input_schema`: `(now, onboarding_session_id, device_id, consent_asserted, max_total_attempts, max_session_enroll_time_ms, lock_after_consecutive_passes)`
- `output_schema`: `Result<VoiceIdEnrollStartResult, StorageError>`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `DECLARED (DB_WRITE)`

### `VID_ENROLL_SAMPLE_COMMIT_ROW`
- `name`: Commit one enrollment sample and update session progress
- `input_schema`: `(now, voice_enrollment_session_id, audio_sample_ref, attempt_index, result, reason_code, idempotency_key)`
- `output_schema`: `Result<VoiceIdEnrollSampleResult, StorageError>`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `DECLARED (DB_WRITE)`

### `VID_ENROLL_COMPLETE_COMMIT_ROW`
- `name`: Finalize enrollment and write voice profile
- `input_schema`: `(now, voice_enrollment_session_id, idempotency_key)`
- `output_schema`: `Result<VoiceIdEnrollCompleteResult, StorageError>`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `DECLARED (DB_WRITE)`

### `VID_ENROLL_DEFER_COMMIT_ROW`
- `name`: Mark enrollment deferred/pending
- `input_schema`: `(now, voice_enrollment_session_id, reason_code, idempotency_key)`
- `output_schema`: `Result<VoiceIdEnrollDeferResult, StorageError>`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `DECLARED (DB_WRITE)`

### `VID_READ_ENROLLMENT_STATE`
- `name`: Read session/sample/profile enrollment state
- `input_schema`: `voice_enrollment_session_id | voice_profile_id`
- `output_schema`: `VoiceEnrollmentSessionRecord | VoiceEnrollmentSampleRecord[] | VoiceProfileRecord`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `NONE`

### `VID_APPEND_ONLY_GUARD`
- `name`: Attempt overwrite sample guard (must fail)
- `input_schema`: `(voice_enrollment_session_id, sample_seq)`
- `output_schema`: `Result<(), StorageError>`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `NONE`

### `VID_ENROLL_COMPLETE_COMMIT_ROW` sync receipt output (implemented)
- `name`: Finalize voice profile and emit deterministic sync receipt ref
- `input_schema`: `(voice_enrollment_session_id, idempotency_key)`
- `output_schema`: `VoiceIdEnrollCompleteResult{voice_profile_id, voice_artifact_sync_receipt_ref}`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `DECLARED (DB_WRITE)`
- `note`: completion now enqueues deterministic `mobile_artifact_sync_queue` rows keyed by `voice_artifact_sync_receipt_ref` so local->cloud sync workers have concrete pending jobs.

## Failure Modes + Reason Codes
- wake context stale/rejected (fail-closed): `VID_FAIL_LOW_CONFIDENCE`
- high-echo risk signal (fail-closed): `VID_FAIL_ECHO_UNSAFE`
- append-only mutation attempt: `VID_APPEND_ONLY_VIOLATION`
- idempotency replay/no-op: `VID_IDEMPOTENCY_REPLAY`
- enrollment session not found/scope mismatch: `VID_SCOPE_VIOLATION`
- contract validation failure: `VID_CONTRACT_VALIDATION_FAILED`
- sync enqueue failure: `VID_SYNC_ENQUEUE_FAILED`
- sync scope violation: `VID_SYNC_SCOPE_VIOLATION`

## Runtime Guardrails (Identity Boundary)
- When `wake_event` is present, identity binding must fail closed unless wake was accepted and remains within the bounded wake binding window.
- If `risk_signals` includes `HIGH_ECHO_RISK`, PH1.VOICE.ID fails closed with echo-unsafe reason code when policy requires it.
- Identity assertion remains non-authoritative: no permissions, no tool calls, no side effects.
- Missing local artifact pointer or unresolved local/cloud pointer conflict must fail closed to `SpeakerAssertionUnknown`.
- Continuous sync failure must not silently proceed as healthy; runtime emits reason-coded degraded sync state for remediation.

## Audit Emission Requirements Per Capability
- Write capabilities must emit PH1.J events with:
  - `event_type`
  - `reason_code`
  - `tenant_id`
  - `user_id`
  - `device_id`
  - `idempotency_key`
- Required minimum event classes:
  - start draft: `STATE_TRANSITION`
  - sample commit: `PERCEPTION_SIGNAL_EMITTED`
  - complete/defer: `STATE_TRANSITION`

## Sources
- `crates/selene_storage/src/repo.rs` (`Ph1VidEnrollmentRepo`)
- `docs/DB_WIRING/PH1_VOICE_ID.md`
