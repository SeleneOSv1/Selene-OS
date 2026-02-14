PH1.REM DB Wiring Spec
1) Engine Header
engine_id: PH1.REM
purpose: Persist deterministic reminder scheduling and timing mechanics (time capture, timezone/DST normalization, recurrence bounds, follow-up timing, retry timing) with a strict split: PH1.REM owns timing mechanics only; message lifecycle/content and interruption policy remain owned by PH1.BCAST (see Section BCAST.MHP.REM in docs/DB_WIRING/PH1_BCAST.md).
version: v1
status: PASS

2) Data Owned (authoritative)
Reminder persistence tables (canonical contract: KC.17 in docs/04_KERNEL_CONTRACTS.md):
reminders
truth_type: CURRENT
primary key: reminder_id
invariants:
state machine is bounded and deterministic
timezone and local_time_mode are stored explicitly
recurrence expansion is bounded and deterministic
idempotency required for retriable writes

reminder_occurrences
truth_type: CURRENT
primary key: occurrence_id
invariants:
one reminder may have multiple occurrences
occurrence_index is deterministic and bounded
state transitions are bounded and deterministic

reminder_delivery_attempts
truth_type: LEDGER
primary key: attempt_id
invariants:
append-only delivery attempt proof history
idempotency required: duplicate delivery_attempt_id is a NOOP
external delivery proofs (if any) are stored as bounded references only

BCAST.MHP follow-up reminder linkage (timing only)
PH1.REM must support reminder_type = BCAST_MHP_FOLLOWUP for the BCAST.MHP lifecycle handoff.
Fields required (either as explicit columns or bounded JSON payload):
tenant_id
user_id (recipient)
broadcast_id
recipient_id
due_at
priority_level
prompt_dedupe_key
idempotency_key tied to (broadcast_id + recipient_id + due_at + prompt_dedupe_key)

3) Reads (dependencies)
reads: identities, devices, sessions (scope/identity checks)
reads: policy snapshots for timezone/quiet-hours interpretation (read-only)
optional read (handoff safety): BCAST recipient state by (broadcast_id, recipient_id) to avoid firing after CONCLUDED (Selene OS may also enforce this precondition; PH1.REM must fail closed if provided state indicates CONCLUDED).

4) Writes (outputs)
All writes are simulation-gated (No Simulation -> No Execution).

REMINDER_SCHEDULE_COMMIT
writes: reminders + reminder_occurrences (and delivery_attempts only when scheduling pre-alert delivery)
idempotency: idempotent on (user_id + scheduled_time + reminder_type + idempotency_key)

REMINDER_UPDATE_COMMIT
writes: reminders (and may adjust next occurrence deterministically)
idempotency: idempotent on (reminder_id + idempotency_key)

REMINDER_CANCEL_COMMIT
writes: reminders state -> CANCELED and cancels pending occurrences/follow-ups deterministically
idempotency: idempotent on (reminder_id + idempotency_key)

REMINDER_SNOOZE_COMMIT
writes: occurrence state + snooze_until deterministically
idempotency: idempotent on (reminder_id + occurrence_id + idempotency_key)

REMINDER_FOLLOWUP_SCHEDULE_COMMIT
writes: occurrence followup_time + state -> FOLLOWUP_PENDING
idempotency: idempotent on (reminder_id + occurrence_id + idempotency_key)

REMINDER_DELIVERY_RETRY_SCHEDULE_COMMIT
writes: occurrence retry_time deterministically
idempotency: idempotent on (reminder_id + occurrence_id + idempotency_key)

REMINDER_DELIVER_PRE_COMMIT / REMINDER_DELIVER_DUE_COMMIT / REMINDER_ESCALATE_COMMIT
writes: reminder_delivery_attempts (append-only) + occurrence state transitions
hard rule: PH1.REM does not decide message content; it only executes timing + delivery attempt proof recording. Content and lifecycle messaging remains in PH1.BCAST for BCAST_MHP_FOLLOWUP.

REMINDER_MARK_COMPLETED_COMMIT / REMINDER_MARK_FAILED_COMMIT
writes: occurrence terminal state and cancels remaining follow-ups deterministically
idempotency: idempotent on (reminder_id + occurrence_id + idempotency_key)

5) Relations & Keys
All records are tenant-scoped.
Reminder records must be replayable and auditable by correlation_id + idempotency_key.
Delivery attempt ledger is append-only.

6) Audit Emissions (PH1.J)
All PH1.REM commit paths emit reason-coded audit events with bounded payload keys:
reminder_id
occurrence_id
reminder_type
priority_level
scheduled_time/due_at (bounded)
delivery_channel (if applicable)
delivery_status (if applicable)

7) Acceptance Tests (Design IDs)
AT-REM-DB-01 tenant isolation enforced
AT-REM-DB-02 append-only enforced for delivery_attempts
AT-REM-DB-03 idempotency dedupe works for schedule/update/cancel
AT-REM-DB-04 recurrence expansion bounded deterministically
AT-REM-DB-05 BCAST_MHP_FOLLOWUP scheduling idempotency key uses (broadcast_id + recipient_id + due_at + prompt_dedupe_key)

Implementation references:
none (design-only; to be implemented in Rust)
