# PH1_ONBOARDING_SMS DB Wiring (Design vNext)

## A) Engine Header
- engine_id: PH1.ONBOARDING_SMS
- layer: Onboarding Control
- authority: Authoritative (SMS setup lifecycle state only)
- role: Ensure SMS app send/receive setup is complete before SMS delivery is allowed
- placement: TURN_OPTIONAL (invoked during onboarding and first SMS send request)

## B) Ownership
Owned tables (design-level):
- comms.sms_app_setup_ledger (append-only)
- comms.sms_app_setup_current (rebuildable)

Required state fields:
- tenant_id
- user_id
- sms_app_setup_complete (bool)
- sms_read_permission_ok (bool)
- sms_send_permission_ok (bool)
- setup_state (PENDING | IN_PROGRESS | COMPLETE | BLOCKED)
- setup_source (ONBOARDING | FIRST_SEND_REQUEST | SETTINGS)
- prompt_dedupe_key
- idempotency_key
- created_at
- updated_at

Rules:
- SMS setup completion is one-time global per user unless permissions are revoked.
- idempotent writes dedupe on (tenant_id, user_id, idempotency_key).
- repeated setup prompts must respect prompt dedupe rules.

## C) Hard Boundaries
- never sends user messages (send path belongs to PH1.BCAST + PH1.DELIVERY).
- never grants authority or bypasses Access + Simulation.
- never calls engines directly; Selene OS orchestrates all calls.
- writes only setup lifecycle state and emits reason-coded outcomes.

## D) Wiring
- Invoked_by: Selene OS onboarding flow and pre-SMS-delivery gate.
- Inputs_from: PH1.X user intent context, PH1.ACCESS gate decision context, device permission probes.
- Outputs_to: Selene OS setup decision (`sms_app_setup_complete`) and next prompt directive.
- Invocation_condition:
  - ALWAYS when delivery_method=SMS and setup is unknown/incomplete.
  - OPTIONAL during proactive onboarding.
- Not allowed:
  - engine-to-engine direct calls
  - external delivery
  - permission authority mutations

## E) Acceptance Tests
- AT-ONBOARDING-SMS-01: If SMS app setup is incomplete, guided setup prompts are emitted deterministically.
- AT-ONBOARDING-SMS-02: After setup completion, future SMS requests do not ask setup again.
- AT-ONBOARDING-SMS-03: SMS onboarding must complete before SMS send commit path is allowed.
