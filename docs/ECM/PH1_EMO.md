# PH1.EMO ECM Spec

## Engine Header
- `engine_id`: `PH1.EMO`
- `purpose`: Deterministic emotional profile and tone guidance lifecycle with strict tone-only guarantees.
- `data_owned`: emotional namespace writes in `preferences_*`, bounded snapshot refs in `artifacts_ledger`, and reason-coded `audit_events`
- `version`: `v1`
- `status`: `ACTIVE`

## Capability List

### `PH1EMO_CLASSIFY_PROFILE_COMMIT_ROW`
- `name`: Classify/lock personality profile (tone-only)
- `input_schema`: `(now, tenant_id, user_id, session_id, consent_asserted, identity_verified, signals, idempotency_key)`
- `output_schema`: `Result<(user_id, personality_type, personality_lock_status, voice_style_profile, reason_code), StorageError>`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `DECLARED (DB_WRITE)`
- `simulation_gated`: `EMO_SIM_001`

### `PH1EMO_REEVALUATE_PROFILE_COMMIT_ROW`
- `name`: Re-evaluate personality profile at deterministic gates
- `input_schema`: `(now, tenant_id, user_id, session_id, consent_asserted, identity_verified, signals_window_ref, idempotency_key)`
- `output_schema`: `Result<(user_id, personality_type, personality_lock_status, reason_code), StorageError>`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `DECLARED (DB_WRITE)`
- `simulation_gated`: `EMO_SIM_002`

### `PH1EMO_PRIVACY_COMMAND_COMMIT_ROW`
- `name`: Apply emotional privacy command
- `input_schema`: `(now, tenant_id, user_id, session_id, privacy_command, target_key?, idempotency_key)`
- `output_schema`: `Result<(user_id, privacy_state, reason_code), StorageError>`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `DECLARED (DB_WRITE)`
- `simulation_gated`: `EMO_SIM_003`

### `PH1EMO_TONE_GUIDANCE_DRAFT_ROW`
- `name`: Emit per-turn tone guidance (output only)
- `input_schema`: `(now, tenant_id, user_id?, profile_snapshot_ref?, signals)`
- `output_schema`: `Result<(tone_guidance, reason_code), StorageError>`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `NONE`
- `simulation_gated`: `EMO_SIM_004`

### `PH1EMO_SNAPSHOT_CAPTURE_COMMIT_ROW`
- `name`: Capture onboarding emotional snapshot
- `input_schema`: `(now, tenant_id, user_id, onboarding_session_id, consent_asserted, identity_verified, signals, idempotency_key)`
- `output_schema`: `Result<(user_id, snapshot_ref, snapshot_status, reason_code), StorageError>`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `DECLARED (DB_WRITE)`
- `simulation_gated`: `EMO_SIM_005`

### `PH1EMO_AUDIT_EVENT_COMMIT_ROW`
- `name`: Emit audit-grade emotional event
- `input_schema`: `(now, tenant_id, user_id, session_id?, event_type, reason_codes, idempotency_key)`
- `output_schema`: `Result<(event_id, status), StorageError>`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `DECLARED (DB_WRITE)`
- `simulation_gated`: `EMO_SIM_006`

## Failure Modes + Reason Codes
- `EMO_FAIL_CONSENT_REQUIRED`
- `EMO_FAIL_IDENTITY_REQUIRED`
- `EMO_FAIL_SCOPE_VIOLATION`
- `EMO_FAIL_IDEMPOTENCY_REPLAY`
- `EMO_FAIL_PRIVACY_COMMAND_INVALID`
- `EMO_FAIL_TONE_ONLY_VIOLATION_BLOCKED`

## Hard Rules
- PH1.EMO is non-authoritative and tone-only.
- PH1.EMO outputs must never alter facts, permissions, or execution decisions.
- PH1.EMO must remain simulation-gated for all commit side effects.

## Sources
- `docs/DB_WIRING/PH1_EMO.md`
- `docs/12_MEMORY_ARCHITECTURE.md` (Q18 emotional boundaries)
