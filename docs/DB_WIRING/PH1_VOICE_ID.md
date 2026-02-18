# PH1.VOICE.ID DB Wiring Spec

## 1) Engine Header

- `engine_id`: `PH1.VOICE.ID`
- `implementation_id`: `PH1.VOICE.ID.001`
- `active_implementation_ids`: `[PH1.VOICE.ID.001]`
- `purpose`: Persist deterministic voice-enrollment session state, append-only enrollment samples, and stable voice profile artifacts for identity binding.
- `version`: `v1`
- `status`: `PASS`

## 2) Data Owned (authoritative)

### `os_core.voice_enrollment_sessions`
- `truth_type`: `CURRENT`
- `primary key`: `voice_enrollment_session_id`
- invariants:
  - FK scope check to `onboarding_sessions.onboarding_session_id` (enforced in PH1.F storage wiring in this slice)
  - FK `device_id -> devices.device_id`
  - `voice_enroll_status` in `IN_PROGRESS | LOCKED | PENDING | DECLINED`
  - `lock_after_consecutive_passes` in `[2, 5]`
  - `max_total_attempts` in `[5, 20]`
  - `max_session_enroll_time_ms` in `[60000, 300000]`
  - idempotent start dedupe on `(onboarding_session_id, device_id)`

### `os_core.voice_enrollment_samples`
- `truth_type`: `LEDGER`
- `primary key`: `sample_id` (in-memory runtime key: `sample_seq` monotonic per session)
- invariants:
  - FK `voice_enrollment_session_id -> voice_enrollment_sessions.voice_enrollment_session_id`
  - `attempt_index > 0`
  - `result in PASS | FAIL`
  - `reason_code` required when `result=FAIL`
  - idempotent append dedupe on `(voice_enrollment_session_id, attempt_index, idempotency_key)`
  - append-only; overwrite/delete prohibited

### `os_core.voice_profiles`
- `truth_type`: `CURRENT`
- `primary key`: `voice_profile_id`
- invariants:
  - unique profile per `(onboarding_session_id, device_id)`
  - created only after enrollment status reaches `LOCKED`

### `os_core.voice_profile_bindings`
- `truth_type`: `CURRENT`
- `primary key`: `(onboarding_session_id, device_id)`
- invariants:
  - FK `voice_profile_id -> voice_profiles.voice_profile_id`
  - one active binding per `(onboarding_session_id, device_id)`

## 3) Reads (dependencies)

### Enrollment session FK checks
- reads: `onboarding_sessions.onboarding_session_id`, `devices.device_id`
- keys/joins used: direct FK existence lookups
- required indices:
  - `onboarding_sessions(onboarding_session_id)`
  - `devices(device_id)`
- scope rules: onboarding-session and device scoped; no cross-session/cross-device writes
- why this read is required: fail-closed before session start

### Enrollment runtime reads
- reads: `voice_enrollment_sessions` by `voice_enrollment_session_id`
- keys/joins used: PK lookup; session+status filter for deterministic transitions
- required indices:
  - `voice_enrollment_sessions(voice_enrollment_session_id)` (PK)
  - `ix_voice_enrollment_sessions_device_status`
- scope rules: one enrollment stream per `(onboarding_session_id, device_id)`
- why this read is required: apply lock/pending rules without ambiguity

### Sample and profile reads
- reads:
  - `voice_enrollment_samples` by `(voice_enrollment_session_id, sample_seq)`
  - `voice_profiles` by `voice_profile_id`
  - `voice_profile_bindings` by `(onboarding_session_id, device_id)`
- keys/joins used: deterministic key lookups
- required indices:
  - `ix_voice_enrollment_samples_session_seq`
  - `ux_voice_profiles_onb_device`
  - `voice_profile_bindings(onboarding_session_id, device_id)` (PK)
- scope rules: per onboarding-session/device pairing
- why this read is required: deterministic completion and retrieval

### Pronunciation robustness hints (related engine boundary)
- reads: bounded pronunciation lexicon hints supplied by Selene OS from `PH1.PRON` (optional)
- keys/joins used: `pack_id` + tenant/user scope checks in OS context
- scope rules:
  - tenant-scoped hints only by default
  - user-scoped hints require explicit consent assertion before use
- why this read is required: improve enrollment/verification robustness without changing identity authority rules

### Wake context + risk hints (non-DB policy/runtime inputs)
- reads: bounded runtime context from Selene OS (`wake_event`, `tts_playback_active`, `risk_signals[]`)
- keys/joins used: n/a (in-memory contract inputs only)
- scope rules:
  - wake context, when present, is used only for bounded identity-window checks
  - risk signals are advisory policy inputs; no authority changes
- why this read is required: deterministic fail-closed identity decisions under stale/rejected wake context and high-echo risk conditions

## 4) Writes (outputs)

### Start enrollment session (draft)
- writes: `voice_enrollment_sessions`
- required fields:
  - `voice_enrollment_session_id`, `onboarding_session_id`, `device_id`, `voice_enroll_status`,
  - `lock_after_consecutive_passes`, `attempt_count`, `max_total_attempts`, `max_session_enroll_time_ms`, `created_at`, `updated_at`
- ledger event_type (if ledger): n/a (`CURRENT` row create/update)
- idempotency_key rule (exact formula):
  - dedupe key = `(onboarding_session_id, device_id)`
- failure reason codes:
  - `VID_ENROLLMENT_REQUIRED` (consent/assertion missing)
  - `VID_FAIL_PROFILE_NOT_ENROLLED` (FK/session prerequisite failure)

### Append enrollment sample (commit)
- writes: `voice_enrollment_samples` + update `voice_enrollment_sessions`
- required fields:
  - sample: `voice_enrollment_session_id`, `sample_seq`, `attempt_index`, `audio_sample_ref`, `result`, `idempotency_key`
  - session update: `attempt_count`, `consecutive_passes`, `voice_enroll_status`, `reason_code`, `updated_at`
- ledger event_type (if ledger): `VOICE_ENROLL_SAMPLE_COMMIT`
- idempotency_key rule (exact formula):
  - dedupe key = `(voice_enrollment_session_id, attempt_index, idempotency_key)`
- failure reason codes:
  - `VID_FAIL_NO_SPEECH`
  - `VID_FAIL_LOW_CONFIDENCE`
  - `VID_FAIL_ECHO_UNSAFE`

### Complete enrollment (commit)
- writes: `voice_enrollment_sessions`, `voice_profiles`, `voice_profile_bindings`
- required fields:
  - `voice_enrollment_session_id`, `idempotency_key`, `updated_at`
  - generated `voice_profile_id`
- ledger event_type (if ledger): n/a (profile creation + binding current rows)
- idempotency_key rule (exact formula):
  - dedupe key = `(voice_enrollment_session_id, idempotency_key)`
- failure reason codes:
  - `VID_FAIL_LOW_CONFIDENCE` (cannot complete before lock)
  - `VID_ENROLLMENT_REQUIRED`

### Defer enrollment reminder (commit)
- writes: `voice_enrollment_sessions`
- required fields:
  - `voice_enrollment_session_id`, `voice_enroll_status=PENDING`, `reason_code`, `updated_at`, `idempotency_key`
- ledger event_type (if ledger): n/a
- idempotency_key rule (exact formula):
  - dedupe key = `(voice_enrollment_session_id, idempotency_key)`
- failure reason codes:
  - `VID_REAUTH_REQUIRED`
  - `VID_ENROLLMENT_REQUIRED`

## 5) Relations & Keys

FKs:
- `voice_enrollment_sessions.device_id -> devices.device_id`
- `voice_enrollment_samples.voice_enrollment_session_id -> voice_enrollment_sessions.voice_enrollment_session_id`
- `voice_profile_bindings.voice_profile_id -> voice_profiles.voice_profile_id`
- `voice_enrollment_sessions.onboarding_session_id -> onboarding_sessions.onboarding_session_id` (enforced in PH1.F storage wiring for this slice)

Unique constraints:
- `voice_enrollment_sessions(voice_enrollment_session_id)` (PK)
- `ux_voice_enrollment_sessions_onb_device`
- `ux_voice_enrollment_samples_session_idempotency`
- `voice_profiles(voice_profile_id)` (PK)
- `ux_voice_profiles_onb_device`
- `voice_profile_bindings(onboarding_session_id, device_id)` (PK)

State machine constraints:
- `IN_PROGRESS -> LOCKED | PENDING`
- `LOCKED -> LOCKED` (idempotent complete)
- `PENDING/DECLINED` cannot accept new sample appends

## 6) Audit Emissions (PH1.J)

PH1.VOICE.ID enrollment writes must emit PH1.J audit events with:
- `event_type`:
  - `VOICE_ENROLL_START_DRAFT`
  - `VOICE_ENROLL_SAMPLE_COMMIT`
  - `VOICE_ENROLL_COMPLETE_COMMIT`
  - `VOICE_ENROLL_DEFER_COMMIT`
- `reason_code(s)`:
  - `VID_FAIL_NO_SPEECH`
  - `VID_FAIL_LOW_CONFIDENCE`
  - `VID_FAIL_ECHO_UNSAFE`
  - `VID_FAIL_PROFILE_NOT_ENROLLED`
  - `VID_ENROLLMENT_REQUIRED`
  - `VID_REAUTH_REQUIRED`
- `payload_min` allowlisted keys:
  - `voice_enrollment_session_id`
  - `onboarding_session_id`
  - `device_id`
  - `voice_enroll_status`
  - `attempt_index`
  - `sample_result`
  - `voice_profile_id`
- `evidence_ref` type:
  - bounded enrollment sample reference only (`audio_sample_ref` / sample seq); no raw audio blob content

## 7) Acceptance Tests (DB Wiring Proof)

- `AT-VID-DB-01` tenant isolation enforced
  - `at_vid_db_01_tenant_isolation_enforced`
- `AT-VID-DB-02` append-only enforcement (no UPDATE/DELETE for ledgers)
  - `at_vid_db_02_append_only_enforced`
- `AT-VID-DB-03` idempotency dedupe works
  - `at_vid_db_03_idempotency_dedupe_works`
- `AT-VID-DB-04` current-table consistency with enrollment sample ledger
  - `at_vid_db_04_current_table_consistency_with_sample_ledger`

Implementation references:
- storage wiring: `crates/selene_storage/src/ph1f.rs`
- typed repo: `crates/selene_storage/src/repo.rs`
- migration: `crates/selene_storage/migrations/0008_ph1vid_voice_enrollment_tables.sql`
- tests: `crates/selene_storage/tests/ph1_voice_id/db_wiring.rs`
