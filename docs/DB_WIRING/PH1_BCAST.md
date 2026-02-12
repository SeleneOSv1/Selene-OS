# PH1.BCAST DB Wiring (Design vNext)

## A) Engine Header
- engine_id: PH1.BCAST
- layer: Control
- authority: Authoritative (broadcast lifecycle state only)
- role: broadcast lifecycle orchestrator (draft -> deliver -> ack/defer -> retry -> close)
- placement: TURN_OPTIONAL (invoked by Selene OS after Access+Simulation when a broadcast is requested)

## B) Ownership (tables owned - design)
Owned tables:
- comms.broadcast_envelopes_ledger (append-only)
- comms.broadcast_envelopes_current (rebuildable)
- comms.broadcast_recipients_current
- comms.broadcast_delivery_attempts_ledger (append-only)
- comms.broadcast_ack_ledger (append-only)

Rules:
- idempotency_key is required for retriable writes.
- per-recipient state machine is persisted in current + ledger views.
- never-ask-twice is enforced using prompt_dedupe_key (persisted per recipient/thread).

## C) Hard Boundaries
- never grants permission or modifies authority state.
- never sends without Access + Simulation gate approval.
- never uses PH1.E.
- never calls PH1.DELIVERY directly (Selene OS orchestrates).
- privacy handshake is required for PRIVATE and CONFIDENTIAL classifications.

## D) Wiring (system accurate)
- Invoked_by: Selene OS (post intent confirm + access decision).
- Inputs_from: PH1.NLP intent_draft + Selene OS policy context + Access decision + blueprint step context.
- Outputs_to: Selene OS lifecycle updates; Selene OS triggers simulations; Selene OS invokes PH1.DELIVERY inside COMMIT steps.
- Invocation_condition: OPTIONAL(broadcast intent or escalation flow).
- Not allowed:
  - engine-to-engine direct calls.
  - execution without simulation context.
  - authority mutation outside Access/override simulations.

## E) Acceptance Tests
- AT-BCAST-01: draft -> access -> simulation -> deliver path is deterministic and auditable.
- AT-BCAST-02: access denial results in zero delivery attempts.
- AT-BCAST-03: defer schedules deterministic retry and prevents duplicate send.
- AT-BCAST-04: never-ask-twice works via prompt dedupe key persistence.
- AT-BCAST-05: classification policy enforcement is deterministic.
- AT-BCAST-06: duplicate idempotency key resolves as NOOP.
