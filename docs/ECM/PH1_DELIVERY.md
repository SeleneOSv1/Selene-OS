# PH1.DELIVERY ECM (Design vNext)

## Engine Header
- engine_id: PH1.DELIVERY
- layer: Control
- authority: Authoritative delivery attempt truth only
- allowed_callers: SELENE_OS_ONLY

## Capability List

### capability_id: DELIVERY_SEND
- input_schema: tenant_id, message_id, recipient, channel, payload_ref, provider, simulation_context, idempotency_key
- output_schema: delivery_attempt_id, delivery_proof_ref, delivery_status, reason_code
- side_effects: EXTERNAL_SEND
- failure_modes: SIMULATION_CONTEXT_MISSING, CHANNEL_UNAVAILABLE, PROVIDER_SEND_FAILED
- reason_codes: DELIVERY_SIMULATION_CONTEXT_MISSING, DELIVERY_CHANNEL_UNAVAILABLE, DELIVERY_PROVIDER_SEND_FAILED

### capability_id: DELIVERY_STATUS
- input_schema: delivery_attempt_id, provider, provider_message_ref
- output_schema: normalized_status, provider_status_raw, reason_code
- side_effects: NONE
- failure_modes: ATTEMPT_NOT_FOUND, PROVIDER_STATUS_UNAVAILABLE
- reason_codes: DELIVERY_ATTEMPT_NOT_FOUND, DELIVERY_PROVIDER_STATUS_UNAVAILABLE

### capability_id: DELIVERY_CANCEL
- input_schema: delivery_attempt_id, provider, simulation_context, idempotency_key
- output_schema: canceled=true|false, reason_code
- side_effects: EXTERNAL_SEND
- failure_modes: SIMULATION_CONTEXT_MISSING, CANCEL_NOT_SUPPORTED, PROVIDER_CANCEL_FAILED
- reason_codes: DELIVERY_SIMULATION_CONTEXT_MISSING, DELIVERY_CANCEL_NOT_SUPPORTED, DELIVERY_PROVIDER_CANCEL_FAILED

### capability_id: DELIVERY_PROVIDER_HEALTH_CHECK
- input_schema: provider, region_hint
- output_schema: provider_health_state, latency_bucket, reason_code
- side_effects: NONE
- failure_modes: PROVIDER_HEALTH_UNAVAILABLE
- reason_codes: DELIVERY_PROVIDER_HEALTH_UNAVAILABLE

## Hard Rules
- DELIVERY_SEND must reject when simulation_context is missing.
- all EXTERNAL_SEND capabilities require simulation gating.
- secrets use KMS handles only; no raw secret material in requests/responses.
