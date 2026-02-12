# PH1.DELIVERY DB Wiring (Design vNext)

## A) Engine Header
- engine_id: PH1.DELIVERY
- layer: Control
- authority: Authoritative (delivery attempt truth only)
- role: provider gateway for SMS/Email/WhatsApp/WeChat, idempotent + auditable
- placement: TURN_OPTIONAL (invoked only by Selene OS during COMMIT delivery steps)

## B) Ownership (tables owned - design)
Owned tables:
- comms.delivery_attempts_ledger (append-only)
- comms.delivery_attempts_current (rebuildable)
- comms.delivery_provider_health (current)

Rules:
- idempotency_key is mandatory: hash(tenant_id + message_id + recipient + channel + payload_hash).
- provider status is deterministically mapped to internal status buckets.

## C) Hard Boundaries
- never decides audience, classification, or privacy policy.
- never bypasses Access + Simulation.
- secrets are KMS handles only; no secret values in payload or audit records.

## D) Wiring
- Invoked_by: Selene OS only, inside a COMMIT simulation step.
- Inputs_from: PH1.BCAST step context (recipient + payload_ref) supplied by Selene OS.
- Outputs_to: Selene OS (delivery_proof_ref + status), then Selene OS updates PH1.BCAST recipient_state.
- Invocation_condition: OPTIONAL(delivery action required).
- Not allowed:
  - direct caller other than Selene OS.
  - provider send without simulation context.
  - mutation of broadcast policy state.

## E) Acceptance Tests
- AT-DELIVERY-01: idempotency prevents duplicate provider sends.
- AT-DELIVERY-02: status polling maps provider states deterministically.
- AT-DELIVERY-03: retryable vs terminal failures are reason-coded.
- AT-DELIVERY-04: secrets are never leaked (handle-only).
