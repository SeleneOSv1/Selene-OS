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

## Section BCAST.MHP: Message Handling Process (Phone-First, Deterministic)

Purpose:
- Define the canonical single-recipient message handling lifecycle for "message to JD" delivery with phone-first behavior and bounded follow-up.

Deterministic state machine:
- `SENT -> WAITING -> FOLLOWUP -> REMINDER_SET -> REMINDER_FIRED -> CONCLUDED`
- Canonical representation: `SENT → WAITING → FOLLOWUP → REMINDER_SET → REMINDER_FIRED → CONCLUDED`

Primary delivery behavior (default):
- Primary channel is always Selene App inbox (phone-first).
- On send, device notifications include:
  - phone vibration
  - desktop email-like notification sound

Urgency notification pattern:
- `NON-URGENT`: normal single notification pattern; default WAITING timeout is 5 minutes.
- `URGENT`: stronger notification pattern with multiple vibration pulses and repeated alert pattern (bounded, deterministic).

Reply handling:
- If JD replies inside Selene App thread (example: "6pm"), Selene OS auto-forwards the resolved reply to wife Selene App and marks thread `CONCLUDED`.
- This app-thread reply path does not trigger Selene voice interruption.
- If JD answers Selene verbally, Selene forwards the answer to wife Selene App and updates lifecycle state to `CONCLUDED` after delivery proof is recorded.

Multiple-attempt behavior:
- After bounded attempts are exhausted, Selene asks one mitigation question and then stops retrying until user decision.
- Exact mitigation question text:
  - "I could not reach JD after multiple attempts. Do you want me to keep trying, send as non-urgent, or stop?"

Fallback order (only when Selene App is unavailable):
1. SMS
2. WhatsApp (outside China)
3. WeChat (in China)
4. Email (last resort)

Policy integration note:
- FOLLOWUP/REMINDER voice interruptions must consult `PH1.POLICY` interruption decision before speaking; phone delivery is already done first.

Additional acceptance tests for BCAST.MHP:
- `AT-BCAST-07`: Single-recipient flow enters `SENT` and transitions to `WAITING` with phone-first delivery proof.
- `AT-BCAST-08`: `NON-URGENT` flow waits exactly 5 minutes before `FOLLOWUP`.
- `AT-BCAST-09`: App-thread reply auto-forwards to wife Selene App and concludes without voice interruption.
- `AT-BCAST-10`: Verbal JD reply is forwarded to wife Selene App and thread concludes deterministically.
- `AT-BCAST-11`: Urgent classification uses stronger/multi-pulse notification profile before follow-up/reminder.
- `AT-BCAST-12`: Fallback routing is used only when Selene App unavailable, in the locked order (SMS -> WhatsApp -> WeChat -> Email).
