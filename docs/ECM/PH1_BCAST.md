# PH1.BCAST ECM (Design vNext)

## Engine Header
- engine_id: PH1.BCAST
- implementation_id: PH1.BCAST.001
- layer: Control
- authority: Authoritative lifecycle control only
- allowed_callers: SELENE_OS_ONLY

## Implementation Lock
- active implementation ids (locked):
  - `PH1.BCAST.001`
- unknown implementation ids must fail closed.

## Capability List

### capability_id: BCAST_DRAFT_CREATE
- input_schema: tenant_id, sender_user_id, audience_spec, classification, content_payload_ref, idempotency_key
- output_schema: broadcast_id, status=draft_created, reason_code
- side_effects: INTERNAL_DB_WRITE
- failure_modes: INPUT_SCHEMA_INVALID, TENANT_SCOPE_INVALID, DUPLICATE_IDEMPOTENCY_KEY
- reason_codes: BCAST_INPUT_SCHEMA_INVALID, BCAST_TENANT_SCOPE_INVALID, BCAST_DUPLICATE_NOOP

### capability_id: BCAST_AUDIENCE_RESOLVE
- input_schema: broadcast_id, audience_spec, policy_context
- output_schema: resolved_recipients, unresolved_recipients, reason_code
- side_effects: NONE
- failure_modes: AUDIENCE_EMPTY, POLICY_BLOCKED
- reason_codes: BCAST_AUDIENCE_EMPTY, BCAST_POLICY_BLOCKED

### capability_id: BCAST_PRIVACY_HANDSHAKE
- input_schema: broadcast_id, recipient_id, classification, privacy_context
- output_schema: privacy_decision, delivery_mode, reason_code
- side_effects: NONE
- failure_modes: PRIVACY_BLOCKED, PRIVACY_CONTEXT_MISSING
- reason_codes: BCAST_PRIVACY_BLOCKED, BCAST_PRIVACY_CONTEXT_MISSING

### capability_id: BCAST_DELIVER_COMMIT
- input_schema: broadcast_id, recipient_id, delivery_plan_ref, recipient_region, app_unavailable, simulation_context, idempotency_key
- output_schema: delivery_request_ref, recipient_state (`WAITING|FOLLOWUP`), followup_immediate, reason_code
- side_effects: EXTERNAL_DELIVERY_REQUEST
- failure_modes: SIMULATION_CONTEXT_MISSING, ACCESS_NOT_ALLOWED, DELIVERY_PLAN_INVALID, FALLBACK_POLICY_VIOLATION
- reason_codes: BCAST_SIMULATION_CONTEXT_MISSING, BCAST_ACCESS_DENIED, BCAST_DELIVERY_PLAN_INVALID, BCAST_FAIL_FALLBACK_POLICY

### capability_id: BCAST_ACK_RECORD
- input_schema: broadcast_id, recipient_id, ack_status, ack_at, idempotency_key
- output_schema: ack_state, reason_code
- side_effects: INTERNAL_DB_WRITE
- failure_modes: RECIPIENT_NOT_FOUND, INVALID_ACK_TRANSITION
- reason_codes: BCAST_RECIPIENT_NOT_FOUND, BCAST_INVALID_ACK_TRANSITION

### capability_id: BCAST_DEFER_AND_SCHEDULE_RETRY
- input_schema: broadcast_id, recipient_id, defer_until, handoff_to_reminder, retry_policy, idempotency_key
- output_schema: retry_scheduled_at, recipient_state (`DEFERRED` or `REMINDER_SET`), reason_code
- side_effects: INTERNAL_DB_WRITE
- failure_modes: INVALID_DEFER_WINDOW, RETRY_POLICY_INVALID
- reason_codes: BCAST_INVALID_DEFER_WINDOW, BCAST_RETRY_POLICY_INVALID

### capability_id: BCAST_REMINDER_FIRED
- input_schema: broadcast_id, recipient_id, reminder_ref, idempotency_key
- output_schema: recipient_state=REMINDER_FIRED, reason_code
- side_effects: INTERNAL_DB_WRITE
- failure_modes: RECIPIENT_NOT_FOUND, INVALID_REMINDER_TRANSITION
- reason_codes: BCAST_RECIPIENT_NOT_FOUND, BCAST_INVALID_REMINDER_TRANSITION

### capability_id: BCAST_ESCALATE_TO_SENDER
- input_schema: broadcast_id, recipient_id, escalation_reason, idempotency_key
- output_schema: escalation_state, sender_notice_ref, reason_code
- side_effects: INTERNAL_DB_WRITE
- failure_modes: ESCALATION_NOT_ALLOWED, SENDER_CONTEXT_MISSING, WAITING_WINDOW_NOT_ELAPSED
- reason_codes: BCAST_ESCALATION_NOT_ALLOWED, BCAST_SENDER_CONTEXT_MISSING, BCAST_FAIL_WAITING_WINDOW_NOT_ELAPSED

### capability_id: BCAST_CANCEL
- input_schema: broadcast_id, cancel_reason, idempotency_key
- output_schema: status=canceled, reason_code
- side_effects: INTERNAL_DB_WRITE
- failure_modes: INVALID_CANCEL_STATE
- reason_codes: BCAST_INVALID_CANCEL_STATE

### capability_id: BCAST_EXPIRE
- input_schema: broadcast_id, expired_at, expiry_reason, idempotency_key
- output_schema: status=expired, reason_code
- side_effects: INTERNAL_DB_WRITE
- failure_modes: INVALID_EXPIRY_STATE
- reason_codes: BCAST_INVALID_EXPIRY_STATE

## Simulation ID Binding (PH1.BCAST.001)
- `BCAST_DRAFT_CREATE` -> simulation_id `BCAST_CREATE_DRAFT`, simulation_type `DRAFT`
- `BCAST_DELIVER_COMMIT` -> simulation_id `BCAST_DELIVER_COMMIT`, simulation_type `COMMIT`
- `BCAST_DEFER_AND_SCHEDULE_RETRY` -> simulation_id `BCAST_DEFER_COMMIT`, simulation_type `COMMIT`
- `BCAST_REMINDER_FIRED` -> simulation_id `BCAST_REMINDER_FIRED_COMMIT`, simulation_type `COMMIT`
- `BCAST_ACK_RECORD` -> simulation_id `BCAST_ACK_COMMIT`, simulation_type `COMMIT`
- `BCAST_ESCALATE_TO_SENDER` -> simulation_id `BCAST_ESCALATE_COMMIT`, simulation_type `COMMIT`
- `BCAST_EXPIRE` -> simulation_id `BCAST_EXPIRE_COMMIT`, simulation_type `COMMIT`
- `BCAST_CANCEL` -> simulation_id `BCAST_CANCEL_COMMIT`, simulation_type `COMMIT`

Hard rule:
- simulation envelope drift (`simulation_id`/`simulation_type`/`capability_id`) must fail closed in OS wiring.

## Hard Rules
- all EXTERNAL_DELIVERY_REQUEST actions are simulation-gated.
- engines never call engines; Selene OS orchestrates.
- PH1.BCAST never grants authority and never bypasses Access.
- for single-recipient message threads, PH1.BCAST applies the BCAST.MHP lifecycle policy (see `docs/DB_WIRING/PH1_BCAST.md` Section BCAST.MHP).
- follow-up communication mode is `VOICE` by default; `TEXT` is allowed only for explicit `USER_REQUESTED_TEXT` or `CANNOT_SPEAK` cases.
- follow-up continuity is mandatory: `subject_ref` must remain bound to the active thread and voice follow-up requires `active_speaker_user_id == recipient_user_id`; otherwise fail closed.
- PH1.DELIVERY executes notification profiles (`NORMAL` vs `URGENT`) as instructed by broadcast envelope delivery hints.

## Implementation References
- kernel contracts: `crates/selene_kernel_contracts/src/ph1bcast.rs`
- engine runtime: `crates/selene_engines/src/ph1bcast.rs`
- os wiring: `crates/selene_os/src/ph1bcast.rs`
