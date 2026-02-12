# PH1.ONBOARDING_SMS ECM (Design vNext)

## Engine Header
- engine_id: PH1.ONBOARDING_SMS
- layer: Control (Onboarding Gate)
- authority: Authoritative (setup lifecycle truth only)
- allowed_callers: SELENE_OS_ONLY
- placement: TURN_OPTIONAL

## Capability List

### capability_id: SMS_SETUP_CHECK
- input_schema:
  - tenant_id
  - user_id
  - device_id (optional)
  - requested_channel (optional; “sms”)
- output_schema:
  - sms_app_setup_complete (bool)
  - sms_read_permission_ok (bool)
  - sms_send_permission_ok (bool)
  - setup_state
  - reason_code
- side_effects: NONE
- failure_modes: INPUT_SCHEMA_INVALID, USER_SCOPE_INVALID
- reason_codes: SMS_SETUP_INPUT_SCHEMA_INVALID, SMS_SETUP_USER_SCOPE_INVALID

### capability_id: SMS_SETUP_PROMPT
- input_schema:
  - tenant_id
  - user_id
  - setup_state
  - prompt_context (bounded)
  - prompt_dedupe_key
- output_schema:
  - prompt_emitted (bool)
  - prompt_variant_id
  - reason_code
- side_effects: NONE
- failure_modes: INPUT_SCHEMA_INVALID, PROMPT_DEDUPE_SUPPRESSED
- reason_codes: SMS_SETUP_INPUT_SCHEMA_INVALID, SMS_SETUP_PROMPT_DEDUPE_SUPPRESSED

### capability_id: SMS_SETUP_CONFIRM
- input_schema:
  - tenant_id
  - user_id
  - sms_read_permission_ok (bool)
  - sms_send_permission_ok (bool)
  - setup_source
  - simulation_context
  - idempotency_key
- output_schema:
  - sms_app_setup_complete (bool)
  - setup_state
  - reason_code
- side_effects: INTERNAL_DB_WRITE
- failure_modes: SIMULATION_CONTEXT_MISSING, PERMISSION_INCOMPLETE, IDENTITY_NOT_VERIFIED
- reason_codes:
  - SMS_SETUP_SIMULATION_CONTEXT_MISSING
  - SMS_SETUP_PERMISSION_INCOMPLETE
  - SMS_SETUP_IDENTITY_NOT_VERIFIED

## Constraints
- Engines never call engines directly; Selene OS orchestrates.
- All state mutation is simulation-gated via SMS_SETUP_SIM.
- This engine is setup-readiness only; it is not a delivery engine.
