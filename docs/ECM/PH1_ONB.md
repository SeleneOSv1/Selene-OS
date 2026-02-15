# PH1.ONB ECM Spec

## Engine Header
- `engine_id`: `PH1.ONB`
- `purpose`: Persist deterministic onboarding session lifecycle for invited onboarding (start, terms, schema-required verification gates, primary-device proof, access create, complete).
- `data_owned`: `onboarding_sessions` runtime state (plus deterministic idempotency indexes in PH1.F for onboarding steps)
- `version`: `v1`
- `status`: `ACTIVE`

## Capability List

### `PH1ONB_SESSION_START_DRAFT_ROW`
- `name`: Start onboarding session from `LINK_OPEN_ACTIVATE` handoff
- `input_schema`: `(token_id, prefilled_context_ref?, tenant_id?, device_fingerprint)`
- `output_schema`: `Result<OnbSessionStartResult{onboarding_session_id,status,next_step,pinned_schema_id,pinned_schema_version,pinned_overlay_set_id,pinned_selector_snapshot,required_verification_gates[]}, StorageError>`
- `allowed_callers`: `SELENE_OS_ONLY` (simulation-gated)
- `side_effects`: `DECLARED (DB_WRITE)`
- `load_rule_note`: ONB session start resolves activated link context by `token_id`, pins schema context, computes `required_verification_gates[]`, then drives one-question clarify from pinned requirements.

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
- `name`: Commit schema-required evidence capture and sender-notification handoff (legacy capability id retained)
- `input_schema`: `(now, onboarding_session_id, photo_blob_ref, sender_user_id, idempotency_key)`
- `output_schema`: `Result<OnbEmployeePhotoCaptureSendResult, StorageError>`
- `allowed_callers`: `SELENE_OS_ONLY` (simulation-gated)
- `side_effects`: `DECLARED (DB_WRITE)`
- `gate_rule`: callable only when pinned `required_verification_gates[]` contains photo evidence gate.

### `PH1ONB_EMPLOYEE_SENDER_VERIFY_COMMIT_ROW`
- `name`: Commit schema-required sender verification decision (legacy capability id retained)
- `input_schema`: `(now, onboarding_session_id, sender_user_id, decision, idempotency_key)`
- `output_schema`: `Result<OnbEmployeeSenderVerifyResult, StorageError>`
- `allowed_callers`: `SELENE_OS_ONLY` (simulation-gated)
- `side_effects`: `DECLARED (DB_WRITE)`
- `gate_rule`: callable only when pinned `required_verification_gates[]` contains sender confirmation gate.

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

### `PH1ONB_BACKFILL_START_DRAFT_ROW`
- `name`: Start deterministic onboarding requirement backfill campaign
- `input_schema`: `(now, actor_user_id, tenant_id, company_id, position_id, schema_version_id, rollout_scope=CurrentAndNew, idempotency_key)`
- `output_schema`: `Result<(campaign_id, state, pending_target_count), StorageError>`
- `allowed_callers`: `SELENE_OS_ONLY` (simulation-gated)
- `side_effects`: `DECLARED (DB_WRITE)`
- `process_guard`: `ONB_REQUIREMENT_BACKFILL` must not be entered for `rollout_scope=NewHiresOnly`.

### `PH1ONB_BACKFILL_NOTIFY_COMMIT_ROW`
- `name`: Commit deterministic backfill recipient notification/progress
- `input_schema`: `(now, campaign_id, tenant_id, recipient_user_id, idempotency_key)`
- `output_schema`: `Result<(campaign_id, recipient_user_id, target_status), StorageError>`
- `allowed_callers`: `SELENE_OS_ONLY` (simulation-gated)
- `side_effects`: `DECLARED (DB_WRITE)`
- `orchestrator_order`: called once per recipient after BCAST delivery and REM scheduling handoff steps.

### `PH1ONB_BACKFILL_COMPLETE_COMMIT_ROW`
- `name`: Commit deterministic onboarding requirement backfill completion state
- `input_schema`: `(now, campaign_id, tenant_id, idempotency_key)`
- `output_schema`: `Result<(campaign_id, state, completed_target_count, total_target_count), StorageError>`
- `allowed_callers`: `SELENE_OS_ONLY` (simulation-gated)
- `side_effects`: `DECLARED (DB_WRITE)`
- `orchestrator_order`: callable only after recipient notify loop reaches deterministic terminal progress set.

## Failure Modes + Reason Codes
- deterministic onboarding failure domains include:
  - link/session scope mismatch
  - required gate not satisfied (terms/verify/device/access)
- required verification gates are schema-derived from pinned schema context; no hardcoded field alias fallback.
- blocked completion when schema-required sender confirmation is not satisfied
  - idempotency replay/no-op
- all failures are fail-closed and reason-coded.

## Audit Emission Requirements Per Capability
- all write capabilities emit PH1.J with deterministic reason codes and bounded payload.
- simulation-bound write paths preserve `simulation_id`, `correlation_id`, `turn_id`, `idempotency_key` linkage.
- read capabilities emit audit only in replay/diagnostic mode.

## Sources
- `crates/selene_storage/src/repo.rs` (`Ph1OnbRepo`)
- `docs/DB_WIRING/PH1_ONB.md`
