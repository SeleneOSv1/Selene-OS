# PH1.ONBOARDING_SMS DB Wiring (Design vNext)

Contract scope note:
- This file defines wiring + persistence contracts only.
- SMS sending is owned by PH1.BCAST + PH1.DELIVERY and is always simulation-gated.

## A) Engine Header
- engine_id: PH1.ONBOARDING_SMS
- layer: Control (Onboarding Gate)
- authority: Authoritative (SMS setup lifecycle state only)
- role: Ensure phone app SMS send/receive setup is complete before any SMS delivery is allowed
- placement: TURN_OPTIONAL (invoked during onboarding and on first SMS send request)

## B) Ownership (tables owned — design)
Owned tables (design-level; PH1.F owns physical schema/migrations):
- comms.sms_app_setup_ledger (append-only truth)
- comms.sms_app_setup_current (rebuildable current view)

Minimum fields (current view):
- tenant_id
- user_id
- device_id (optional; setup may be per-device depending on policy)
- sms_app_setup_complete (bool)
- sms_read_permission_ok (bool)
- sms_send_permission_ok (bool)
- setup_state (PENDING | IN_PROGRESS | COMPLETE | BLOCKED)
- setup_source (ONBOARDING | FIRST_SEND_REQUEST | SETTINGS)
- prompt_dedupe_key (bounded)
- last_prompted_at (timestamp)
- idempotency_key (for retriable writes)
- created_at
- updated_at

Ledger invariants:
- append-only (no in-place updates)
- idempotent dedupe on (tenant_id, user_id, idempotency_key)

Behavioral invariants:
- sms_app_setup_complete is one-time per user unless permissions are revoked.
- prompt repetition must follow global prompt dedupe policy (WorkOrder prompt_dedupe_keys_json).

## C) Hard Boundaries
- Never sends SMS or messages (send path belongs to PH1.BCAST + PH1.DELIVERY).
- Never grants authority or bypasses Access + Simulation.
- Never calls engines directly; Selene OS orchestrates all calls.
- Writes only SMS setup lifecycle truth and emits reason-coded readiness outcomes.

## D) Wiring
- Invoked_by:
  - Selene OS onboarding flow
  - Selene OS pre-send gate when delivery_method = SMS
- Inputs_from:
  - Selene OS WorkOrder fields (delivery_method, requester_user_id, tenant_id)
  - Device permission probe results (OS/UI layer)
  - Access decision context (for policy constraints only)
- Outputs_to:
  - Selene OS setup readiness: sms_app_setup_complete + setup_state
  - Selene OS next prompt directive (setup prompt vs proceed)
- Invocation_condition:
  - ALWAYS when delivery_method = SMS and setup is unknown/incomplete
  - OPTIONAL during onboarding (proactive configuration)
- Not allowed:
  - engine-to-engine direct calls
  - external delivery actions
  - permission/role mutation (only Access override simulations can do that)

## E) Acceptance Tests
- AT-ONBOARDING-SMS-01: If SMS setup is incomplete, Selene deterministically prompts for required permissions.
- AT-ONBOARDING-SMS-02: After setup completion, future SMS sends do not re-run setup prompts (no-repeat).
- AT-ONBOARDING-SMS-03: SMS send commit path is blocked until sms_app_setup_complete=true (fail closed).
