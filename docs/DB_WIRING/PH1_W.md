# PH1.W DB Wiring Spec

## 1) Engine Header

- `engine_id`: `PH1.W`
- `implementation_id`: `PH1.W.001`
- `active_implementation_ids`: `[PH1.W.001]`
- `purpose`: Persist deterministic wake enrollment lifecycle, enrollment sample quality ledger, runtime wake accept/reject/suppress events, and active wake profile bindings.
- `version`: `v1`
- `status`: `PASS`

## 1A) Phone-First Artifact Custody (Required Extension)

Operating model lock:
- `PH1.W` wake runtime is phone-first for low-latency wake detection.
- Platform trigger policy is explicit: `IOS` defaults to explicit-trigger-only (`explicit_trigger_only=true`, side-button/app-open trigger), while `ANDROID` and desktop may run always-on wake when policy allows.
- Wake artifacts must be stored locally on phone (`ACTIVE + N-1 rollback`) and continuously synced to Selene for continuity/recovery.
- Engine B owns outbox/vault replay and ack semantics; PH1.W owns deterministic wake artifact-manifest delta emission.

Local artifact minimums (phone):
- wake phrase set/version package.
- wake threshold/cooldown package.
- per-device calibration package.
- active wake profile pointer + rollback pointer.

Sync model (mandatory):
- every local wake artifact change emits a sync delta envelope.
- outbox replay continues until ack; deletion is ack-gated.
- raw wake audio remains excluded from default persistence/sync path.

## 2) Data Owned (authoritative)

### `os_core.wake_enrollment_sessions`
- `truth_type`: `CURRENT`
- `primary key`: `wake_enrollment_session_id`
- invariants:
  - FK `user_id -> identities.user_id`
  - FK `device_id -> devices.device_id`
  - optional `onboarding_session_id` FK is enforced in PH1.F storage wiring in this slice
  - `wake_enroll_status` in `IN_PROGRESS | PENDING | COMPLETE | DECLINED`
  - `pass_target` in `[3, 8]`
  - `max_attempts` in `[8, 20]`
  - `attempt_count <= max_attempts`
  - `enrollment_timeout_ms` in `[180000, 600000]`
  - one active in-progress session per `(user_id, device_id)`

### `os_core.wake_enrollment_samples`
- `truth_type`: `LEDGER`
- `primary key`: `sample_id` (runtime deterministic key: session + `sample_seq`)
- invariants:
  - FK `wake_enrollment_session_id -> wake_enrollment_sessions.wake_enrollment_session_id`
  - `sample_seq` monotonic per session
  - `result` in `PASS | FAIL`
  - idempotent append dedupe on `(wake_enrollment_session_id, idempotency_key)`
  - append-only; overwrite/delete prohibited

### `os_core.wake_runtime_events`
- `truth_type`: `LEDGER`
- `primary key`: `wake_event_id`
- invariants:
  - FK `device_id -> devices.device_id`
  - optional FK `session_id -> sessions.session_id`
  - optional FK `user_id -> identities.user_id`
  - idempotent append dedupe on `(device_id, idempotency_key)`
  - append-only; overwrite/delete prohibited

### `os_core.wake_profile_bindings`
- `truth_type`: `CURRENT`
- `primary key`: `(user_id, device_id, wake_profile_id)`
- invariants:
  - FK `user_id -> identities.user_id`
  - FK `device_id -> devices.device_id`
  - `artifact_version` identifies the deterministic wake parameter/tuning package (thresholds, hold frames, cooldown) applied for that binding
  - at most one active binding per `(user_id, device_id)`

## 3) Reads (dependencies)

### Enrollment FK checks
- reads: `identities.user_id`, `devices.device_id`, optional `onboarding_sessions.onboarding_session_id`
- keys/joins used: direct FK existence lookups
- required indices:
  - `identities(user_id)` (PK)
  - `devices(device_id)` (PK)
  - onboarding session key enforced in storage wiring for this lock slice
- scope rules: user/device scoped; no cross-user device enrollment
- why this read is required: fail closed before enrollment writes

### Runtime FK checks
- reads: `devices.device_id`, optional `sessions.session_id`, optional `identities.user_id`
- keys/joins used: direct FK existence lookups
- required indices:
  - `devices(device_id)` (PK)
  - `sessions(session_id)` (PK)
  - `identities(user_id)` (PK)
- scope rules: wake runtime event is device-scoped
- why this read is required: deterministic runtime event validity and no orphan rows

### Runtime policy context inputs (non-DB)
- reads: bounded `WakePolicyContext` from Selene OS
  - `session_state`
  - `do_not_disturb`
  - `privacy_mode`
  - `tts_playback_active`
  - `media_playback_active`
  - `explicit_trigger_only`
- keys/joins used: n/a (in-memory contract input only)
- scope rules:
  - policy context may disarm wake, but cannot assert identity or authority
  - `explicit_trigger_only=true` suppresses wake and must remain reason-coded/auditable
- why this read is required: deterministic fail-closed wake disarm/suppression behavior

### Wake state reads
- reads:
  - `wake_enrollment_sessions` by `wake_enrollment_session_id`
  - `wake_enrollment_samples` by session
  - `wake_runtime_events` by device and time
  - `wake_profile_bindings` by `(user_id, device_id)`
- keys/joins used: deterministic key lookups and ordered runtime reads
- required indices:
  - `ux_wake_enrollment_sessions_active_user_device`
  - `ux_wake_enrollment_samples_session_idempotency`
  - `ux_wake_runtime_events_device_idempotency`
  - `ux_wake_profile_bindings_active_user_device`
- scope rules: no cross-user wake profile lookups
- why this read is required: deterministic dedupe, enrollment progression, and active profile retrieval

### Device-local wake artifact pointer + sync cursor (required extension)
- reads:
  - local active/rollback wake artifact pointers from app runtime context.
  - last synced cursor/receipt refs from Engine B handoff context.
- keys/joins used: `(tenant_id, user_id, device_id, artifact_type)` deterministic key tuple.
- scope rules:
  - no cross-user/cross-device wake pointer reads.
  - unresolved pointer conflict fails closed to conservative wake policy.
- why this read is required: deterministic phone-first wake with recovery-safe cloud continuity.

### Pronunciation robustness hints (related engine boundary)
- reads: bounded pronunciation lexicon hints supplied by Selene OS from `PH1.PRON` (optional)
- keys/joins used: `pack_id` + tenant/user scope checks in OS context
- scope rules:
  - tenant-scoped hints only by default
  - user-scoped hints require explicit consent assertion before use
- why this read is required: improve wake phrase variant robustness without changing wake authority boundaries

## 4) Writes (outputs)

### Start wake enrollment (draft)
- writes: `wake_enrollment_sessions`
- required fields:
  - `wake_enrollment_session_id`, `user_id`, `device_id`, `wake_enroll_status`, `pass_target`,
    `pass_count`, `attempt_count`, `max_attempts`, `enrollment_timeout_ms`, `created_at`, `updated_at`
- platform gate:
  - if `onboarding_session_id` resolves to `app_platform=IOS`, start must fail closed by default (explicit-trigger-only policy).
- idempotency_key rule (exact formula):
  - dedupe key = `(user_id, device_id, idempotency_key)`
- failure reason codes (minimum examples):
  - `W_ENROLL_INVALID_BOUNDS`
  - `W_ENROLL_DEVICE_OWNERSHIP_MISMATCH`
  - `W_ENROLL_ALREADY_IN_PROGRESS`

### Append enrollment sample (commit)
- writes: `wake_enrollment_samples` + update `wake_enrollment_sessions`
- required fields:
  - sample: `wake_enrollment_session_id`, `sample_seq`, `captured_at`, `result`, `idempotency_key`
  - session update: `attempt_count`, `pass_count`, `wake_enroll_status`, `reason_code`, `updated_at`
- idempotency_key rule (exact formula):
  - dedupe key = `(wake_enrollment_session_id, idempotency_key)`
- failure reason codes (minimum examples):
  - `W_ENROLL_SAMPLE_INVALID`
  - `W_ENROLL_SAMPLE_SESSION_CLOSED`

### Complete enrollment (commit)
- writes: `wake_enrollment_sessions` + `wake_profile_bindings`
- required fields:
  - `wake_enrollment_session_id`, `wake_profile_id`, `updated_at`, `completed_at`, `idempotency_key`
  - generated `wake_artifact_sync_receipt_ref` (consumed by `ONB_COMPLETE_COMMIT` gate when wake enrollment is complete)
- idempotency_key rule (exact formula):
  - dedupe key = `(wake_enrollment_session_id, idempotency_key)`
- failure reason codes (minimum examples):
  - `W_ENROLL_PASS_TARGET_NOT_MET`

### Append wake runtime event
- writes: `wake_runtime_events`
- required fields:
  - `wake_event_id`, `device_id`, `created_at`, `accepted`, `reason_code`, `idempotency_key`
  - runtime gate snapshots persisted in row scope: `tts_active_at_trigger`, `media_playback_active_at_trigger`, `suppression_reason_code?`
  - runtime policy snapshot persisted in row scope: `explicit_trigger_only_at_trigger`
  - runtime gate snapshots persisted in row scope: `g1a_utterance_start_ok`, `g3a_liveness_ok`
  - runtime parameter/tuning snapshots persisted in PH1.J payload scope:
    - `parameter_set_id`
    - `enter_threshold`
    - `exit_threshold`
    - `hold_frames`
    - `cooldown_ms`
- idempotency_key rule (exact formula):
  - dedupe key = `(device_id, idempotency_key)`
- failure reason codes (minimum examples):
  - `W_RUNTIME_DEVICE_MISSING`
  - `W_RUNTIME_SESSION_INVALID`
  - `W_RUNTIME_USER_INVALID`

### Enqueue wake artifact-manifest sync delta (commit; future extension)
- writes: Engine B outbox handoff envelope (PH1.W-owned payload contract)
- required fields:
  - `tenant_id`, `user_id`, `device_id`, `engine_id=PH1.W`
  - `artifact_type`, `artifact_version`, `artifact_status`
  - `package_hash`, `payload_ref`, `provenance_ref`
  - `active_pointer_ref`, `rollback_pointer_ref`
  - `consent_scope_ref`, `idempotency_key`
- idempotency_key rule (exact formula):
  - dedupe key = `(tenant_id, user_id, device_id, artifact_type, artifact_version, idempotency_key)`
- failure reason codes (minimum examples):
  - `W_SYNC_ENQUEUE_FAILED`
  - `W_SYNC_SCOPE_VIOLATION`
  - `W_SYNC_PAYLOAD_INVALID`

## 5) Relations & Keys

FKs:
- `wake_enrollment_sessions.user_id -> identities.user_id`
- `wake_enrollment_sessions.device_id -> devices.device_id`
- `wake_enrollment_samples.wake_enrollment_session_id -> wake_enrollment_sessions.wake_enrollment_session_id`
- `wake_runtime_events.device_id -> devices.device_id`
- `wake_runtime_events.session_id -> sessions.session_id` (nullable)
- `wake_runtime_events.user_id -> identities.user_id` (nullable)
- `wake_profile_bindings.user_id -> identities.user_id`
- `wake_profile_bindings.device_id -> devices.device_id`

Unique constraints:
- `wake_enrollment_sessions(wake_enrollment_session_id)` (PK)
- `ux_wake_enrollment_sessions_active_user_device`
- `wake_enrollment_samples(sample_id)` (PK)
- `ux_wake_enrollment_samples_session_idempotency`
- `ux_wake_enrollment_samples_session_seq`
- `wake_runtime_events(wake_event_id)` (PK)
- `ux_wake_runtime_events_device_idempotency`
- `wake_profile_bindings(user_id, device_id, wake_profile_id)` (PK)
- `ux_wake_profile_bindings_active_user_device`

State/boundary constraints:
- wake sample/runtime ledgers are append-only
- wake enrollment session state follows deterministic transitions (`IN_PROGRESS` -> `PENDING|COMPLETE|DECLINED`)
- wake runtime state machine is represented by reason-coded runtime rows:
  - `DISARMED -> ARMED_IDLE -> CANDIDATE -> CONFIRMED -> CAPTURE -> COOLDOWN -> ARMED_IDLE`
  - any state may transition to `SUSPENDED` on audio integrity failure and only return after deterministic stabilization
- wake session start/stop behavior is reason-coded:
  - start: first `accepted=true` `wake_runtime_events` row for a request window
  - stop: `CAPTURE` completion or suppression/reject row that closes that request window
- raw wake audio is not persisted by default in this lock slice

## 6) Audit Emissions (PH1.J)

PH1.W writes must emit PH1.J audit events with:
- `event_type`:
  - `WAKE_ENROLL_START_DRAFT`
  - `WAKE_ENROLL_SAMPLE_COMMIT`
  - `WAKE_ENROLL_COMPLETE_COMMIT`
  - `WAKE_ENROLL_DEFER_COMMIT`
  - `WAKE_RUNTIME_EVENT_COMMIT`
  - `WAKE_ARTIFACT_SYNC_ENQUEUE_COMMIT`
- `reason_code(s)`:
  - `FAIL_G0_DEVICE_UNHEALTHY`
  - `FAIL_G1A_NOT_UTTERANCE_START`
  - `FAIL_G3_SCORE_LOW`
  - `FAIL_G3A_REPLAY_SUSPECTED`
  - `SUPPRESS_EXPLICIT_TRIGGER_ONLY`
  - `SUPPRESS_COOLDOWN`
  - `SUPPRESS_POLICY_SUSPENDED`
  - `W_SYNC_ENQUEUE_FAILED`
- `payload_min` allowlisted keys:
  - `wake_enrollment_session_id`
  - `wake_event_id`
  - `user_id`
  - `device_id`
  - `wake_enroll_status`
  - `sample_seq`
  - `result`
  - `wake_profile_id`
  - `accepted`
  - `suppression_reason_code`
  - `artifact_type`
  - `artifact_version`
  - `active_pointer_ref`
  - `rollback_pointer_ref`

## 7) Acceptance Tests (DB Wiring Proof)

- `AT-W-01` tenant isolation enforced
  - `at_w_db_01_tenant_isolation_enforced`
- `AT-W-02` append-only enforcement for wake ledgers
  - `at_w_db_02_append_only_enforced`
- `AT-W-03` idempotency dedupe works
  - `at_w_db_03_idempotency_dedupe_works`
- `AT-W-04` current-state consistency with enrollment/runtime ledgers
  - `at_w_db_04_current_table_consistency_with_enrollment_and_runtime_ledger`
- `AT-W-05` phone-local wake pointer + cloud sync cursor reconciliation is deterministic
  - `at_w_db_05_phone_local_pointer_sync_cursor_reconcile`
- `AT-W-06` wake artifact sync enqueue is idempotent and ack-gated
  - `at_w_db_06_artifact_sync_enqueue_idempotent_ack_gated`

Implementation references:
- storage wiring: `crates/selene_storage/src/ph1f.rs`
- typed repo: `crates/selene_storage/src/repo.rs`
- migration: `crates/selene_storage/migrations/0011_ph1w_wake_tables.sql`
- tests: `crates/selene_storage/tests/ph1_w/db_wiring.rs`
