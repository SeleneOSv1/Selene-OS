# PH1.ONB ECM Spec

## Engine Header
- `engine_id`: `PH1.ONB`
- `purpose`: Persist deterministic onboarding session lifecycle for invited onboarding (start, terms, employee verification gates, primary-device proof, access create, complete).
- `data_owned`: `onboarding_sessions` runtime state (plus deterministic idempotency indexes in PH1.F for onboarding steps)
- `version`: `v1`
- `status`: `ACTIVE`

## Capability List

### `PH1ONB_SESSION_START_DRAFT_ROW`
- `name`: Start onboarding session from activated link
- `input_schema`: `(now, token_id?, draft_id, prefilled_context_ref?, tenant_id?, device_fingerprint)`
- `output_schema`: `Result<OnbSessionStartResult, StorageError>`
- `allowed_callers`: `SELENE_OS_ONLY` (simulation-gated)
- `side_effects`: `DECLARED (DB_WRITE)`

### `PH1ONB_SESSION_ROW`
- `name`: Read one onboarding session row
- `input_schema`: `onboarding_session_id`
- `output_schema`: `Option<OnboardingSessionRecord>`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `NONE`

### `PH1ONB_SESSION_ROWS`
- `name`: Read all onboarding session rows (replay/testing)
- `input_schema`: `none`
- `output_schema`: `Map<OnboardingSessionId, OnboardingSessionRecord>`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `NONE`

### `PH1ONB_TERMS_ACCEPT_COMMIT_ROW`
- `name`: Commit terms acceptance/decline
- `input_schema`: `(now, onboarding_session_id, terms_version_id, accepted, idempotency_key)`
- `output_schema`: `Result<OnbTermsAcceptResult, StorageError>`
- `allowed_callers`: `SELENE_OS_ONLY` (simulation-gated)
- `side_effects`: `DECLARED (DB_WRITE)`

### `PH1ONB_EMPLOYEE_PHOTO_CAPTURE_SEND_COMMIT_ROW`
- `name`: Commit employee photo capture and sender-notification handoff
- `input_schema`: `(now, onboarding_session_id, photo_blob_ref, sender_user_id, idempotency_key)`
- `output_schema`: `Result<OnbEmployeePhotoCaptureSendResult, StorageError>`
- `allowed_callers`: `SELENE_OS_ONLY` (simulation-gated)
- `side_effects`: `DECLARED (DB_WRITE)`

### `PH1ONB_EMPLOYEE_SENDER_VERIFY_COMMIT_ROW`
- `name`: Commit sender verification decision
- `input_schema`: `(now, onboarding_session_id, sender_user_id, decision, idempotency_key)`
- `output_schema`: `Result<OnbEmployeeSenderVerifyResult, StorageError>`
- `allowed_callers`: `SELENE_OS_ONLY` (simulation-gated)
- `side_effects`: `DECLARED (DB_WRITE)`

### `PH1ONB_PRIMARY_DEVICE_CONFIRM_COMMIT_ROW`
- `name`: Commit primary-device proof outcome
- `input_schema`: `(now, onboarding_session_id, device_id, proof_type, proof_ok, idempotency_key)`
- `output_schema`: `Result<OnbPrimaryDeviceConfirmResult, StorageError>`
- `allowed_callers`: `SELENE_OS_ONLY` (simulation-gated)
- `side_effects`: `DECLARED (DB_WRITE)`

### `PH1ONB_ACCESS_INSTANCE_CREATE_COMMIT_ROW`
- `name`: Commit access-instance creation linkage
- `input_schema`: `(now, onboarding_session_id, user_id, tenant_id?, role_id, idempotency_key)`
- `output_schema`: `Result<OnbAccessInstanceCreateResult, StorageError>`
- `allowed_callers`: `SELENE_OS_ONLY` (simulation-gated)
- `side_effects`: `DECLARED (DB_WRITE)`

### `PH1ONB_COMPLETE_COMMIT_ROW`
- `name`: Commit onboarding completion
- `input_schema`: `(now, onboarding_session_id, idempotency_key)`
- `output_schema`: `Result<OnbCompleteResult, StorageError>`
- `allowed_callers`: `SELENE_OS_ONLY` (simulation-gated)
- `side_effects`: `DECLARED (DB_WRITE)`

## Failure Modes + Reason Codes
- deterministic onboarding failure domains include:
  - link/session scope mismatch
  - required gate not satisfied (terms/verify/device/access)
  - blocked employee completion without sender confirmation
  - idempotency replay/no-op
- all failures are fail-closed and reason-coded.

## Audit Emission Requirements Per Capability
- all write capabilities emit PH1.J with deterministic reason codes and bounded payload.
- simulation-bound write paths preserve `simulation_id`, `correlation_id`, `turn_id`, `idempotency_key` linkage.
- read capabilities emit audit only in replay/diagnostic mode.

## Sources
- `crates/selene_storage/src/repo.rs` (`Ph1OnbRepo`)
- `docs/DB_WIRING/PH1_ONB.md`
