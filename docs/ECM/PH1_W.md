# PH1.W ECM Spec

## Engine Header
- `engine_id`: `PH1.W`
- `implementation_id`: `PH1.W.001`
- `active_implementation_ids`: `[PH1.W.001]`
- `purpose`: Persist deterministic wake enrollment/session/runtime lifecycle and active wake profile bindings under append-only/idempotent rules.
- `data_owned`: `wake_enrollment_sessions`, `wake_enrollment_samples`, `wake_runtime_events`, `wake_profile_bindings`
- `version`: `v1`
- `status`: `ACTIVE`
- `related_inputs`: Optional pronunciation-hint packs from `PH1.PRON` for wake-variant robustness only (no wake authority changes)

## Capability List

### `PH1W_ENROLL_START_DRAFT_ROW`
- `name`: Start wake enrollment draft session
- `input_schema`: `(now, user_id, device_id, onboarding_session_id?, pass_target, max_attempts, enrollment_timeout_ms, idempotency_key)`
- `output_schema`: `Result<WakeEnrollmentSessionRecord, StorageError>`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `DECLARED (DB_WRITE)`
- `simulation_ids`: `[WAKE_ENROLL_START_DRAFT]`

### `PH1W_ENROLL_SAMPLE_COMMIT_ROW`
- `name`: Commit one wake enrollment sample and update enrollment counters/state
- `input_schema`: `(now, wake_enrollment_session_id, sample quality fields, result, reason_code?, idempotency_key)`
- `output_schema`: `Result<WakeEnrollmentSessionRecord, StorageError>`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `DECLARED (DB_WRITE)`
- `simulation_ids`: `[WAKE_ENROLL_SAMPLE_COMMIT]`

### `PH1W_ENROLL_COMPLETE_COMMIT_ROW`
- `name`: Complete wake enrollment and bind active wake profile
- `input_schema`: `(now, wake_enrollment_session_id, wake_profile_id, artifact_version, idempotency_key)`
- `output_schema`: `Result<WakeEnrollmentSessionRecord, StorageError>`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `DECLARED (DB_WRITE)`
- `simulation_ids`: `[WAKE_ENROLL_COMPLETE_COMMIT]`

### `PH1W_ENROLL_DEFER_COMMIT_ROW`
- `name`: Mark wake enrollment deferred/pending with deterministic reason code
- `input_schema`: `(now, wake_enrollment_session_id, deferred_until?, reason_code, idempotency_key)`
- `output_schema`: `Result<WakeEnrollmentSessionRecord, StorageError>`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `DECLARED (DB_WRITE)`
- `simulation_ids`: `[WAKE_ENROLL_DEFER_COMMIT]`

### `PH1W_RUNTIME_EVENT_COMMIT_ROW`
- `name`: Commit wake runtime accept/reject/suppress event
- `input_schema`: `(now, wake_event_id, session_id?, user_id?, device_id, accepted, reason_code, wake_profile_id?, tts_active_at_trigger, media_playback_active_at_trigger, explicit_trigger_only_at_trigger, g1a_utterance_start_ok, g3a_liveness_ok, suppression_reason_code?, state_from, state_to, parameter_set_id?, enter_threshold?, exit_threshold?, hold_frames?, cooldown_ms?, idempotency_key)`
- `output_schema`: `Result<WakeRuntimeEventRecord, StorageError>`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `DECLARED (DB_WRITE)`

### `PH1W_READ_ENROLLMENT_SESSION_ROW`
- `name`: Read one wake enrollment session row
- `input_schema`: `wake_enrollment_session_id`
- `output_schema`: `Option<WakeEnrollmentSessionRecord>`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `NONE`

### `PH1W_READ_ENROLLMENT_SAMPLE_ROWS`
- `name`: Read wake enrollment sample ledger for a session
- `input_schema`: `wake_enrollment_session_id`
- `output_schema`: `WakeEnrollmentSampleRecord[]`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `NONE`

### `PH1W_READ_RUNTIME_EVENT_ROWS`
- `name`: Read wake runtime ledger rows
- `input_schema`: `none`
- `output_schema`: `WakeRuntimeEventRecord[]`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `NONE`

### `PH1W_READ_ACTIVE_WAKE_PROFILE`
- `name`: Read active wake profile binding for `(user_id, device_id)`
- `input_schema`: `(user_id, device_id)`
- `output_schema`: `Option<wake_profile_id>`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `NONE`

### `PH1W_APPEND_ONLY_GUARDS`
- `name`: Guard against overwrite of wake sample/runtime ledgers
- `input_schema`: `(wake_enrollment_session_id, sample_seq) | wake_event_id`
- `output_schema`: `Result<(), StorageError>`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `NONE`

## Failure Modes + Reason Codes
- enrollment bound/quality validation failure: `W_ENROLL_INVALID_BOUNDS`, `W_ENROLL_SAMPLE_INVALID`
- enrollment state/ownership violation: `W_ENROLL_ALREADY_IN_PROGRESS`, `W_ENROLL_SAMPLE_SESSION_CLOSED`, `W_ENROLL_DEVICE_OWNERSHIP_MISMATCH`
- runtime scope validation failure: `W_RUNTIME_DEVICE_MISSING`, `W_RUNTIME_SESSION_INVALID`, `W_RUNTIME_USER_INVALID`
- runtime state transition validation failure: `W_RUNTIME_STATE_TRANSITION_INVALID`
- explicit-trigger-only suppression: `W_SUPPRESS_EXPLICIT_TRIGGER_ONLY`
- start-of-utterance gate rejection: `W_FAIL_G1A_NOT_UTTERANCE_START`
- liveness/replay rejection: `W_FAIL_G3A_REPLAY_SUSPECTED`
- idempotency replay/no-op: `W_IDEMPOTENCY_REPLAY`

## Audit Emission Requirements Per Capability
- Enrollment/runtime write capabilities must emit PH1.J events with deterministic reason codes:
  - `WAKE_ENROLL_START_DRAFT`
  - `WAKE_ENROLL_SAMPLE_COMMIT`
  - `WAKE_ENROLL_COMPLETE_COMMIT`
  - `WAKE_ENROLL_DEFER_COMMIT`
  - `WAKE_RUNTIME_EVENT_COMMIT`
- Runtime reject/suppress rows must preserve gate/suppression reason code taxonomy in bounded payload.
- Runtime rows must preserve state transition and tuning snapshots (`state_from`, `state_to`, `parameter_set_id`, threshold/hysteresis fields) in bounded payload.
- Runtime rows must preserve explicit policy/gate snapshots needed for replay (`explicit_trigger_only_at_trigger`, `g1a_utterance_start_ok`, `g3a_liveness_ok`).
- Read/guard capabilities emit audit only in explicit replay/diagnostic mode.

## Runtime Guardrails (Wake Boundary)
- Unknown `implementation_id` must fail closed at contract parse/runtime dispatch boundaries.
- `explicit_trigger_only=true` is a suppression policy signal only; it must never execute wake capture.
- Wake remains non-authoritative: no identity/permission/action side effects.

## Sources
- `crates/selene_storage/src/repo.rs` (`Ph1wWakeRepo`)
- `docs/DB_WIRING/PH1_W.md`
