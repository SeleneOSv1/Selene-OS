# PH1.VOICE.ID ECM Spec

## Engine Header
- `engine_id`: `PH1.VOICE.ID`
- `purpose`: Persist voice enrollment session/sample/profile lifecycle under deterministic idempotency and append-only sample history.
- `data_owned`: `voice_enrollment_sessions`, `voice_enrollment_samples`, `voice_profiles`
- `version`: `v1`
- `status`: `ACTIVE`

## Capability List

### `VID_ENROLL_START_DRAFT_ROW`
- `name`: Start enrollment session draft
- `input_schema`: `(now, onboarding_session_id, device_id, consent_asserted, max_total_attempts, max_session_enroll_time_ms, lock_after_consecutive_passes)`
- `output_schema`: `Result<VoiceEnrollmentSessionRecord, StorageError>`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `DECLARED (DB_WRITE)`

### `VID_ENROLL_SAMPLE_COMMIT_ROW`
- `name`: Commit one enrollment sample and update session progress
- `input_schema`: `(now, voice_enrollment_session_id, audio_sample_ref, attempt_index, result, reason_code, idempotency_key)`
- `output_schema`: `Result<VoiceEnrollmentSessionRecord, StorageError>`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `DECLARED (DB_WRITE)`

### `VID_ENROLL_COMPLETE_COMMIT_ROW`
- `name`: Finalize enrollment and write voice profile
- `input_schema`: `(now, voice_enrollment_session_id, idempotency_key)`
- `output_schema`: `Result<VoiceEnrollmentSessionRecord, StorageError>`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `DECLARED (DB_WRITE)`

### `VID_ENROLL_DEFER_REMINDER_COMMIT_ROW`
- `name`: Mark enrollment deferred/pending
- `input_schema`: `(now, voice_enrollment_session_id, reason_code, idempotency_key)`
- `output_schema`: `Result<VoiceEnrollmentSessionRecord, StorageError>`
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

## Failure Modes + Reason Codes
- append-only mutation attempt: `VID_APPEND_ONLY_VIOLATION`
- idempotency replay/no-op: `VID_IDEMPOTENCY_REPLAY`
- enrollment session not found/scope mismatch: `VID_SCOPE_VIOLATION`
- contract validation failure: `VID_CONTRACT_VALIDATION_FAILED`

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
