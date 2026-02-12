# PH1_ONBOARDING_SMS ECM (Design vNext)

## Engine Header
- engine_id: PH1.ONBOARDING_SMS
- role: SMS app setup verification and completion workflow
- placement: TURN_OPTIONAL

## Capability List

### capability_id: SMS_SETUP_CHECK
- input_schema: tenant_id, user_id, device_id(optional), requested_channel(optional)
- output_schema: sms_app_setup_complete, sms_read_permission_ok, sms_send_permission_ok, setup_state, reason_code
- allowed_callers: SELENE_OS_ONLY
- side_effects: NONE
- failure_modes: INPUT_SCHEMA_INVALID, USER_SCOPE_INVALID
- reason_codes: SMS_SETUP_INPUT_SCHEMA_INVALID, SMS_SETUP_USER_SCOPE_INVALID

### capability_id: SMS_SETUP_PROMPT
- input_schema: tenant_id, user_id, setup_state, prompt_context, prompt_dedupe_key
- output_schema: prompt_emitted(bool), prompt_variant_id, reason_code
- allowed_callers: SELENE_OS_ONLY
- side_effects: NONE
- failure_modes: INPUT_SCHEMA_INVALID, PROMPT_DEDUPE_SUPPRESSED
- reason_codes: SMS_SETUP_INPUT_SCHEMA_INVALID, SMS_SETUP_PROMPT_DEDUPE_SUPPRESSED

### capability_id: SMS_SETUP_CONFIRM
- input_schema: tenant_id, user_id, sms_read_permission_ok, sms_send_permission_ok, setup_source, simulation_context, idempotency_key
- output_schema: sms_app_setup_complete, setup_state, reason_code
- allowed_callers: SELENE_OS_ONLY
- side_effects: INTERNAL_DB_WRITE
- failure_modes: SIMULATION_CONTEXT_MISSING, PERMISSION_INCOMPLETE, IDENTITY_NOT_VERIFIED
- reason_codes: SMS_SETUP_SIMULATION_CONTEXT_MISSING, SMS_SETUP_PERMISSION_INCOMPLETE, SMS_SETUP_IDENTITY_NOT_VERIFIED

## Constraints
- Engines never call engines directly; Selene OS orchestrates.
- All state mutation is simulation-gated (`SMS_SETUP_SIM`).
- This engine is non-delivery; it only validates setup readiness and persists setup truth.
