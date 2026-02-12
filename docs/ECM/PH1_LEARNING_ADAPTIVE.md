# PH1_LEARNING_ADAPTIVE ECM (Design vNext)

## Engine Header
- engine_id: PH1.LEARNING_ADAPTIVE
- role: User feedback and multilingual usage adaptation hints
- placement: TURN_OPTIONAL (async)

## Capability List

### capability_id: LEARN_DRAFT_FEEDBACK
- input_schema: tenant_id, user_id, draft_ref, correction_ref, correction_kind, simulation_context, idempotency_key
- output_schema: feedback_event_id, quality_delta_bucket, reason_code
- allowed_callers: SELENE_OS_ONLY
- side_effects: INTERNAL_DB_WRITE
- failure_modes: INPUT_SCHEMA_INVALID, SIMULATION_CONTEXT_MISSING, DUPLICATE_NOOP
- reason_codes: LEARN_INPUT_SCHEMA_INVALID, LEARN_SIMULATION_CONTEXT_MISSING, LEARN_DUPLICATE_NOOP

### capability_id: LEARN_LANGUAGE_USAGE
- input_schema: tenant_id, user_id, language_segments, preferred_response_language, correction_refs(optional), simulation_context, idempotency_key
- output_schema: usage_event_id, language_profile_delta, reason_code
- allowed_callers: SELENE_OS_ONLY
- side_effects: INTERNAL_DB_WRITE
- failure_modes: INPUT_SCHEMA_INVALID, SIMULATION_CONTEXT_MISSING, PROFILE_SCOPE_INVALID
- reason_codes: LEARN_INPUT_SCHEMA_INVALID, LEARN_SIMULATION_CONTEXT_MISSING, LEARN_PROFILE_SCOPE_INVALID

## Constraints
- Engines never call engines directly; Selene OS orchestrates.
- Learning writes are non-authoritative and simulation-gated (`LEARN_MODEL_UPDATE_SIM`).
- Capability outputs may influence future ranking hints only; they cannot alter permissions or execution policy.
- multilingual model/corpus training is offline (for example, mT5-class multilingual adaptation) and surfaces only as bounded hint artifacts.
