# PH1_EMO_CORE ECM (Design vNext)

## Engine Header
- engine_id: PH1.EMO.CORE
- implementation_id: PH1.EMO.CORE.001
- role: Deterministic emotional profile/snapshot/tone guidance core
- placement: TURN_OPTIONAL

## Capability List

### capability_id: PH1EMO_CLASSIFY_PROFILE_COMMIT_ROW
- simulation_id: `EMO_SIM_001` (COMMIT)
- input_schema:
  - `tenant_id`, `requester_user_id`, optional `session_id`
  - `consent_asserted=true`
  - `identity_verified=true`
  - bounded `signals` bundle
  - `idempotency_key`
- output_schema:
  - `personality_type` (`PASSIVE | DOMINEERING | UNDETERMINED`)
  - `personality_lock_status`
  - `voice_style_profile`
  - `reason_code`
- allowed_callers: SELENE_OS_ONLY
- side_effects: NONE

### capability_id: PH1EMO_REEVALUATE_PROFILE_COMMIT_ROW
- simulation_id: `EMO_SIM_002` (COMMIT)
- input_schema:
  - `tenant_id`, `requester_user_id`
  - `consent_asserted=true`
  - `identity_verified=true`
  - `signals_window_ref`
  - `idempotency_key`
- output_schema:
  - reevaluated `personality_type`
  - reevaluated `personality_lock_status`
  - `reason_code`
- allowed_callers: SELENE_OS_ONLY
- side_effects: NONE

### capability_id: PH1EMO_PRIVACY_COMMAND_COMMIT_ROW
- simulation_id: `EMO_SIM_003` (COMMIT)
- input_schema:
  - `tenant_id`, `requester_user_id`
  - `identity_verified=true`
  - `privacy_command`
  - `confirmation_asserted` (required for destructive commands)
  - optional `target_key` (required for `FORGET_THIS_KEY`)
  - `idempotency_key`
- output_schema:
  - `privacy_state`
  - `reason_code`
- allowed_callers: SELENE_OS_ONLY
- side_effects: NONE

### capability_id: PH1EMO_TONE_GUIDANCE_DRAFT_ROW
- simulation_id: `EMO_SIM_004` (DRAFT)
- input_schema:
  - `tenant_id`
  - optional `requester_user_id`
  - optional `profile_snapshot_ref`
  - bounded `signals`
  - `idempotency_key`
- output_schema:
  - `tone_guidance` (`style_profile_ref`, ordered modifiers, pacing/directness/empathy levels)
  - `reason_code`
- allowed_callers: SELENE_OS_ONLY
- side_effects: NONE

### capability_id: PH1EMO_SNAPSHOT_CAPTURE_COMMIT_ROW
- simulation_id: `EMO_SIM_005` (COMMIT)
- input_schema:
  - `tenant_id`, `requester_user_id`
  - `onboarding_session_id`
  - `consent_asserted`, `identity_verified`
  - bounded `signals`
  - `idempotency_key`
- output_schema:
  - `snapshot_status` (`COMPLETE | DEFER`)
  - optional `snapshot_ref`
  - `reason_code`
- allowed_callers: SELENE_OS_ONLY
- side_effects: NONE (result packet only; persistence is externalized)

### capability_id: PH1EMO_AUDIT_EVENT_COMMIT_ROW
- simulation_id: `EMO_SIM_006` (COMMIT)
- input_schema:
  - `tenant_id`, `requester_user_id`
  - optional `session_id`
  - `event_type`, `reason_codes`
  - `idempotency_key`
- output_schema:
  - deterministic `event_id`
  - `status=RECORDED`
- allowed_callers: SELENE_OS_ONLY
- side_effects: NONE (emits deterministic audit packet only)

## Failure Modes + Reason Codes
- `PH1_EMO_CORE_FAIL_SCHEMA_INVALID`
- `PH1_EMO_CORE_FAIL_CONSENT_REQUIRED`
- `PH1_EMO_CORE_FAIL_IDENTITY_REQUIRED`
- `PH1_EMO_CORE_FAIL_PRIVACY_CONFIRMATION_REQUIRED`
- `PH1_EMO_CORE_FAIL_SCOPE_VIOLATION`
- `PH1_EMO_CORE_FAIL_INTERNAL`

## Constraints
- Non-authoritative boundary is mandatory: PH1.EMO.CORE cannot grant permissions, execute actions, or reorder gates.
- Tone-only contract is mandatory: outputs must carry `tone_only=true`, `no_meaning_drift=true`, `no_execution_authority=true`.
- Runtime behavior is deterministic for identical inputs.
- Simulation-id/capability drift must fail closed in OS wiring.

## Related Boundaries
- PH1.EMO.CORE is a concrete implementation surface and is invoked directly by Selene OS/blueprints.
- PH1.EMO.GUIDE remains a separate concrete tone-policy assist surface.
- PH1.X/PH1.TTS consume PH1.EMO.CORE outputs as advisory only.
