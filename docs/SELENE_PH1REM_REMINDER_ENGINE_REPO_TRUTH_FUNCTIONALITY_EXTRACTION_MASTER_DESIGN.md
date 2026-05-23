# Selene PH1.REM Reminder Engine — Repo-Truth Functionality Extraction Master Design

DOCUMENT STATUS:
REPO_TRUTH_EXTRACTION
NOT_RUNTIME_IMPLEMENTATION
PENDING_GRAND_ARCHITECTURE_RECONCILIATION

## 0. Authority and Scope

AGENTS.md controls execution.

This is a docs-only repo-truth extraction.

No runtime code was changed.

This document does not authorize implementation.

The document reconstructs current reminder design/functionality from repo evidence.

Future implementation/refactor/retirement requires explicit build instruction, approved file scope, tests, backend evidence, JD approval, and where visible JD live proof.

This extraction separates:

- FOUND behavior that is present in current repo code, tests, docs, or scripts.
- PARTIAL behavior that exists in one layer but is not fully wired end to end.
- NOT_FOUND behavior that was searched for but not located as current repo truth.
- DESIGN_GAP / TEST_GAP / OWNER_GAP / AUDIT_GAP where the repo shows missing or underdefined architecture.

## 1. Executive Summary

PH1.REM is Selene's governed reminder timing boundary in current repo truth.

It exists as:

- a kernel contract in `crates/selene_kernel_contracts/src/ph1rem.rs`;
- an OS runtime wrapper in `crates/selene_os/src/ph1rem.rs`;
- PH1.F in-memory/storage-backed reminder state in `crates/selene_storage/src/ph1f.rs`;
- SimulationExecutor routing for reminder set/update/cancel, calendar-draft reminder creation, and BCAST.MHP follow-up handoff in `crates/selene_os/src/simulation_executor.rs`;
- docs and acceptance scripts declaring PH1.REM as timing-only and preventing it from owning broadcast/delivery content.

PH1.REM is not currently a standalone engine module in `crates/selene_engines/src/ph1rem.rs`; that path was not found. Current active runtime appears to be OS plus PH1.F storage.

Product functions supported by code include:

- schedule reminder;
- update reminder;
- cancel reminder;
- snooze occurrence;
- schedule follow-up;
- schedule retry;
- deliver pre reminder proof;
- deliver due reminder proof;
- escalate reminder to another allowed channel;
- mark reminder completed;
- mark reminder failed;
- generate bounded daily/weekly recurrence occurrences;
- preserve idempotency for schedule/update/delivery attempts;
- support BCAST.MHP follow-up reminders through an OS bridge;
- create a meeting reminder as part of calendar-draft simulation, without writing to an external calendar.

PH1.REM does not own:

- message body wording;
- PH1.WRITE final user-facing phrasing;
- external SMS/email/push provider sends;
- PH1.BCAST broadcast/message lifecycle;
- PH1.DELIVERY provider-attempt truth outside reminder attempt records;
- onboarding session state;
- task/scheduler/roster business state;
- access grants;
- authority or protected execution decisions;
- Desktop/iPhone runtime decisions;
- Adapter business/provider decisions.

Active vs partial vs unclear:

- Active: PH1.REM contract, OS runtime, PH1.F storage state, deterministic time parsing for a limited set of formats, daily/weekly recurrence, state transitions, BCAST.MHP handoff, simulation routing, and tests.
- Partial: database docs describe SQL-like tables and PH1.J audit, but no migration file was found; delivery attempt proof exists, but live provider delivery is not proven; onboarding reminder/follow-up is documented and future-designed, but active postpone/resume onboarding proof was not found.
- Unclear / missing: full timezone/DST handling, quiet hours computation, PH1.WRITE reminder wording boundary, GPT-5.5 natural-language time parsing, live notification providers, task/scheduler/roster integration, SQL persistence migrations, and JD live acceptance.

## 2. Repo Evidence Inventory

| Evidence Area | File / Path | Symbols / Routes / Tables | Status | Notes |
| --- | --- | --- | --- | --- |
| Contract | `crates/selene_kernel_contracts/src/ph1rem.rs` | `PH1REM_ENGINE_ID`, `PH1REM_IMPLEMENTATION_ID`, `ReminderRequest`, `Ph1RemRequest`, `Ph1RemOk`, `Ph1RemRefuse` | FOUND | Canonical schema and simulation-id validation. |
| Simulation IDs | `crates/selene_kernel_contracts/src/ph1rem.rs` | `REMINDER_SCHEDULE_COMMIT`, `REMINDER_UPDATE_COMMIT`, `REMINDER_CANCEL_COMMIT`, `REMINDER_SNOOZE_COMMIT`, `REMINDER_DELIVER_PRE_COMMIT`, `REMINDER_DELIVER_DUE_COMMIT`, `REMINDER_FOLLOWUP_SCHEDULE_COMMIT`, `REMINDER_ESCALATE_COMMIT`, `REMINDER_DELIVERY_RETRY_SCHEDULE_COMMIT`, `REMINDER_MARK_COMPLETED_COMMIT`, `REMINDER_MARK_FAILED_COMMIT` | FOUND | Request variant must match envelope simulation id. |
| Reminder enums | `crates/selene_kernel_contracts/src/ph1rem.rs` | `ReminderType`, `ReminderChannel`, `ReminderState`, `ReminderDeliveryStatus`, `ReminderPriorityLevel`, `ReminderLocalTimeMode`, `ReminderAckSource` | FOUND | Defines current public state vocabulary. |
| Runtime wrapper | `crates/selene_os/src/ph1rem.rs` | `Ph1RemRuntime::run`, `Ph1RemRuntime::run_for_implementation`, `REM_FAIL_TIME_AMBIGUOUS_NEEDS_CONFIRM` | FOUND | Runtime validates implementation id and delegates to PH1.F. |
| Standalone engine module | `crates/selene_engines/src/ph1rem.rs` | N/A | NOT_FOUND | No separate `selene_engines` PH1.REM module located. |
| Storage/runtime state | `crates/selene_storage/src/ph1f.rs` | `ReminderRecord`, `ReminderOccurrenceRecord`, `ReminderDeliveryAttemptRecord`, `ph1rem_run`, `ph1rem_schedule`, `ph1rem_update`, `ph1rem_cancel`, `ph1rem_snooze`, `ph1rem_followup`, `ph1rem_retry`, `ph1rem_deliver_pre`, `ph1rem_deliver_due`, `ph1rem_escalate`, `ph1rem_mark_completed`, `ph1rem_mark_failed` | FOUND | Main current implementation surface. |
| PH1.F storage getters | `crates/selene_storage/src/ph1f.rs` | `reminders`, `reminder_occurrences`, `reminder_delivery_attempts`, `reminder_row`, `reminder_occurrence_row`, `reminder_delivery_attempt_row` | FOUND | Read surfaces used by tests and app-ingress read-only listing. |
| SQL migrations | `crates/selene_storage/migrations` | `reminders`, `reminder_occurrences`, `reminder_delivery_attempts` | NOT_FOUND | No migration containing PH1.REM table names was located. |
| Generic repo facade | `crates/selene_storage/src/repo.rs` | PH1.REM-specific typed repo | NOT_FOUND | No `ph1rem` match in `repo.rs`. |
| Simulation routing | `crates/selene_os/src/simulation_executor.rs` | `execute_rem`, `SetReminder`, `UpdateReminder`, `CancelReminder`, `CreateCalendarEvent`, `run_broadcast_mhp_defer_with_reminder`, `run_broadcast_mhp_mark_reminder_fired`, `enforce_reminder_mutation_gate` | FOUND | Routes user-intent simulation outcomes into PH1.REM and bridges BCAST.MHP. |
| App ingress wording/listing | `crates/selene_os/src/app_ingress.rs` | `response_text_for_dispatch_outcome`, `maybe_build_list_reminders_tool_response`, `is_list_reminders_query` | FOUND | Hardcoded user-facing reminder wording and read-only reminder listing. |
| Adapter tests | `crates/selene_adapter/src/lib.rs` | `at_adapter_03ba_calendar_event_confirm_yes_dispatches_sim_and_persists_meeting_reminder`, `at_adapter_03bb_cancel_reminder_confirm_yes_dispatches_sim_and_cancels_row`, `at_adapter_03bc_list_reminders_uses_read_only_tool_lane_with_provenance` | FOUND | Confirms adapter path can dispatch and list through OS/runtime, not local authority. |
| HTTP adapter | `crates/selene_adapter/src/bin/http_adapter.rs` | reminder-specific route | NOT_FOUND | No dedicated reminder HTTP provider route located. |
| Desktop client | `apple/mac_desktop` | System Activity / Needs Attention surfaces | PARTIAL | Reminder-specific authority not found; operational rendering surfaces exist. |
| iPhone client | `apple/iphone` | `broadcast_waiting_followup_reminder_state`, System Activity / Needs Attention surfaces | PARTIAL | Reminder status display evidence, not reminder state ownership. |
| DB wiring doc | `docs/DB_WIRING/PH1_REM.md` | `reminders`, `reminder_occurrences`, `reminder_delivery_attempts`, PH1.REM ownership rules | FOUND / PARTIAL | Declares PASS and timing ownership; stale note says Rust implementation design-only despite current code. |
| ECM doc | `docs/ECM/PH1_REM.md` | PH1.REM capabilities, failure modes, allowed callers `SELENE_OS_ONLY` | FOUND | Good owner-boundary evidence. |
| Blueprint | `docs/BLUEPRINTS/REMINDER_MANAGE.md` | `REMINDER_MANAGE` | FOUND | Defines schedule/update/cancel/snooze/follow-up/retry/escalation blueprint. |
| Simulation catalog | `docs/08_SIMULATION_CATALOG.md` | REMINDER simulation rows and detailed sections; `BCAST_REMINDER_FIRED_COMMIT` | FOUND | Catalog registration for active and draft handoffs. |
| Blueprint registry | `docs/09_BLUEPRINT_REGISTRY.md` | `REMINDER_MANAGE` | FOUND | Reminder manage blueprint is registered active. |
| Ownership matrix | `docs/10_DB_OWNERSHIP_MATRIX.md` | PH1.REM owns reminder tables | FOUND | Confirms timing mechanics truth ownership. |
| OS constitution | `docs/05_OS_CONSTITUTION.md` | PH1.REM timing-only statement | FOUND | Reinforces PH1.BCAST message lifecycle split. |
| Boundary script | `scripts/check_delivery_ownership_boundaries.sh` | Guards PH1.REM from directly calling PH1.BCAST/PH1.DELIVERY | FOUND | Strong wrong-owner protection evidence. |
| BCAST MHP script | `scripts/check_bcast_mhp_acceptance.sh` | `at_bcast_mhp_05_reminder_set_and_fired_flow_via_ph1_rem` | FOUND | Acceptance script expects BCAST reminder-fired proof. |

## 3. Current Reminder Owner Map

| Responsibility | Current Owner / File | Correct Future Owner Hypothesis | Status | Notes |
| --- | --- | --- | --- | --- |
| reminder creation | PH1.REM contract/runtime/storage: `ph1rem_schedule` | PH1.REM | FOUND | Requires identity, parses desired time, creates reminder and occurrences. |
| reminder update | PH1.REM storage: `ph1rem_update` | PH1.REM | FOUND | Updates mutable fields and rebuilds occurrences when time/recurrence changes. |
| reminder cancellation | PH1.REM storage: `ph1rem_cancel` | PH1.REM | FOUND | Marks reminder and pending occurrences canceled. |
| reminder snooze | PH1.REM storage: `ph1rem_snooze` | PH1.REM | FOUND | Occurrence scoped, moves scheduled/snooze window. |
| reminder recurrence | PH1.REM storage: `parse_reminder_recurrence_plan`, `generate_reminder_occurrences` | PH1.REM | PARTIAL | Daily/weekly bounded recurrence only. |
| occurrence generation | PH1.REM storage | PH1.REM | FOUND | Max 365 generated occurrences. |
| due firing | PH1.REM storage: `ph1rem_deliver_due` | PH1.REM timing proof plus Delivery/BCAST for external send | PARTIAL | Records delivery attempt proof; no live provider send. |
| pre-due firing | PH1.REM storage: `ph1rem_deliver_pre` | PH1.REM timing proof plus Delivery/BCAST for external send | PARTIAL | Appends delivered attempt proof only. |
| delivery attempt proof | PH1.REM storage: `ReminderDeliveryAttemptRecord` | PH1.REM for reminder-attempt evidence; PH1.DELIVERY for provider truth | PARTIAL | Internal proof ref is stored; provider live proof not found. |
| retry scheduling | PH1.REM storage: `ph1rem_retry`, due-delivery retry branch | PH1.REM for timing; PH1.DELIVERY for provider retry | FOUND / PARTIAL | Retry time stored; no live delivery retry owner proof. |
| follow-up scheduling | PH1.REM storage: `ph1rem_followup`, due-delivery follow-up branch | PH1.REM | FOUND | Sets `FollowupPending` and follow-up timestamp. |
| escalation | PH1.REM storage: `ph1rem_escalate` | PH1.REM timing/evidence with policy owner validation | PARTIAL | Requires prior attempt and allowed channel; broader escalation policy limited. |
| mark completed | PH1.REM storage: `ph1rem_mark_completed` | PH1.REM | FOUND | Terminal state. |
| mark failed | PH1.REM storage: `ph1rem_mark_failed` | PH1.REM | FOUND | Terminal state with failure reason. |
| quiet hours / deferred delivery | Contract enum `DeferredQuietHours`; docs failure modes | PH1.REM plus policy owner | PARTIAL | No active quiet-hours computation located. |
| ambiguous time handling | `resolve_reminder_desired_time`, OS reason code | PH1.REM detects; PH1.WRITE explains | FOUND / PARTIAL | Refuses ambiguous time; no full clarification loop in PH1.REM. |
| timezone handling | Contract/storage fields `user_timezone`, `local_time_mode` | PH1.REM plus language/profile owner | PARTIAL | Stored and validated, but no real timezone/DST conversion located. |
| reminder notification | PH1.REM attempt proof; BCAST/DELIVERY future boundary | BCAST/DELIVERY/clients render; PH1.REM timing only | PARTIAL | Reminder channels exist, live notification providers not proven. |
| BCAST reminder fired resume | Selene OS bridge in `simulation_executor.rs`; BCAST owns lifecycle | Selene OS orchestrates; BCAST owns broadcast state; PH1.REM owns timing | FOUND | PH1.REM does not call BCAST directly. |
| onboarding postponement reminder | ONB docs and BCAST/REM extraction references | PH1.ONB hands off to PH1.REM; BCAST/DELIVERY notify | PARTIAL / DESIGN_GAP | Active end-to-end postpone/resume proof not found. |
| link follow-up reminder | PH1.LINK journey docs mention future status | PH1.LINK status plus PH1.REM timing | NOT_FOUND / DESIGN_GAP | No direct link reminder implementation found. |
| task/scheduler handoff | SimulationExecutor SetReminder and CalendarDraftCreated | PH1.REM timing; Task/Scheduler/Roster own business state | PARTIAL | No task/roster mutation integration found. |
| Desktop rendering | Desktop System Activity surfaces | Desktop render-only | PARTIAL | No local reminder decision authority found. |
| iPhone rendering | iPhone System Activity / Needs Attention | iPhone render-only | PARTIAL | No local reminder decision authority found. |
| Adapter transport | Adapter tests via voice/turn path | Adapter transport only | PARTIAL | No dedicated reminder provider route found. |
| audit/provenance | PH1.F rows, proof refs, docs declare PH1.J | PH1.REM + PH1.J audit | PARTIAL / AUDIT_GAP | Storage proof exists; PH1.J event emission not proven in code. |
| storage/migrations | PH1.F in-memory/storage fields; docs declare DB tables | PH1.F/SQL storage owner | PARTIAL | No migration located. |
| old compatibility paths | BCAST.MHP, app-ingress read-only listing, hardcoded wording | Preserve until proof-based retirement | PARTIAL | Some heuristic/hardcoded paths remain. |

## 4. Current Reminder Lifecycle

### Stage 1 — Create / Schedule Reminder

- Owner: PH1.REM.
- Symbols/files: `ReminderScheduleCommitRequest`, `ReminderRequest::Schedule`, `Ph1RemRuntime::run`, `ph1rem_schedule`, `generate_reminder_occurrences`.
- Inputs: tenant id, user id, optional device id, reminder type, request text, desired time text, user timezone, local time mode, priority, optional recurrence rule, channel preferences, idempotency key.
- Outputs: `Ph1RemOk` with `REMINDER_SCHEDULED`, reminder id, primary occurrence id, state `Scheduled`, scheduled time.
- State changes: creates `ReminderRecord`, one or more `ReminderOccurrenceRecord`s, idempotency indexes, lookup indexes.
- Audit evidence: PH1.F rows and proof refs; PH1.J audit emission not proven.
- Gaps: limited time parser, limited recurrence, no PH1.WRITE guidance ownership, no live notification provider.

### Stage 2 — Update Reminder

- Owner: PH1.REM.
- Symbols/files: `ReminderUpdateFields`, `ReminderUpdateCommitRequest`, `ph1rem_update`.
- Inputs: tenant/user/reminder id, optional fields, idempotency key.
- Outputs: `Ph1RemOk` with `REMINDER_UPDATED`.
- State changes: updates reminder fields and `updated_at`; rebuilds generated occurrences if desired time or recurrence changes.
- Audit evidence: row state changes; audit event emission not proven.
- Gaps: no user-facing preview/confirmation inside PH1.REM; confirmation belongs upstream.

### Stage 3 — Cancel Reminder

- Owner: PH1.REM.
- Symbols/files: `ReminderCancelCommitRequest`, `ph1rem_cancel`.
- Inputs: tenant/user/reminder id, reason, idempotency key.
- Outputs: `Ph1RemOk` with `REMINDER_CANCELED`, state `Canceled`.
- State changes: reminder and scheduled/snoozed/follow-up occurrences become canceled.
- Audit evidence: row state changes.
- Gaps: no visible cancellation notification proof.

### Stage 4 — Snooze Occurrence

- Owner: PH1.REM.
- Symbols/files: `ReminderSnoozeCommitRequest`, `ph1rem_snooze`.
- Inputs: tenant/user/reminder id, occurrence id, snooze-until timestamp, reason, idempotency key.
- Outputs: `Ph1RemOk` with `REMINDER_SNOOZED`, occurrence id, state `Snoozed`, scheduled time.
- State changes: occurrence scheduled time and `snooze_until` updated; reminder state becomes `Snoozed`.
- Audit evidence: row state changes.
- Gaps: no natural-language snooze parse beyond upstream intent draft.

### Stage 5 — Pre-Due Delivery

- Owner: PH1.REM for timing/proof; delivery channel owner not live-proven.
- Symbols/files: `ReminderDeliverPreCommitRequest`, `ph1rem_deliver_pre`, `ph1rem_deliver_common`.
- Inputs: tenant/user/reminder id, occurrence id, channel, scheduled delivery time, proof ref, idempotency key.
- Outputs: `Ph1RemOk` with `REMINDER_DELIVERED_PRE`, delivery status `Delivered`, delivery attempt id, proof ref.
- State changes: append-only delivery attempt row.
- Audit evidence: delivery attempt row and proof ref.
- Gaps: no provider send call; no PH1.DELIVERY integration proof.

### Stage 6 — Due Delivery

- Owner: PH1.REM for timing/proof.
- Symbols/files: `ReminderDeliverDueCommitRequest`, `ph1rem_deliver_due`.
- Inputs: tenant/user/reminder id, occurrence id, channel, fired-at time, proof ref, optional offline state, idempotency key.
- Outputs:
  - delivered branch: `REMINDER_DELIVERED_DUE`, `Delivered`, delivery attempt id, proof ref, state `FollowupPending`;
  - retry branch: `REMINDER_RETRY_SCHEDULED`, `RetryScheduled`, retry time;
  - missed branch: delivered proof plus tighter follow-up time.
- State changes: append attempt row, set follow-up pending or retry time.
- Audit evidence: delivery attempt row and occurrence fields.
- Gaps: deferred quiet-hours enum exists but active quiet-hours computation was not found.

### Stage 7 — Retry Scheduling

- Owner: PH1.REM timing.
- Symbols/files: `ReminderDeliveryRetryScheduleCommitRequest`, `ph1rem_retry`.
- Inputs: tenant/user/reminder id, occurrence id, failed attempt id, retry-at timestamp, channel, reason, idempotency key.
- Outputs: `REMINDER_RETRY_SCHEDULED`, delivery status `RetryScheduled`.
- State changes: occurrence `retry_time` updated.
- Audit evidence: occurrence field.
- Gaps: actual provider retry orchestration is not proven.

### Stage 8 — Follow-Up Scheduling

- Owner: PH1.REM timing.
- Symbols/files: `ReminderFollowupScheduleCommitRequest`, `ph1rem_followup`.
- Inputs: tenant/user/reminder id, occurrence id, follow-up time, reason, idempotency key.
- Outputs: `REMINDER_FOLLOWUP_SCHEDULED`, state `FollowupPending`.
- State changes: reminder and occurrence become follow-up pending.
- Audit evidence: row fields.
- Gaps: follow-up notification delivery is not live-proven.

### Stage 9 — Escalation

- Owner: PH1.REM evidence/timing plus policy owner.
- Symbols/files: `ReminderEscalateCommitRequest`, `ph1rem_escalate`.
- Inputs: tenant/user/reminder id, occurrence id, escalation level, from channel, to channel, reason, proof ref, idempotency key.
- Outputs: `REMINDER_ESCALATED`, delivery status `Delivered`, attempt id, escalation level.
- State changes: delivered attempt appended; occurrence follows pending state.
- Audit evidence: attempt row and escalation level in response.
- Gaps: broad emergency/escalation policy validation is limited to allowed channel/prior attempt checks.

### Stage 10 — Completion / Failure

- Owner: PH1.REM.
- Symbols/files: `ReminderMarkCompletedCommitRequest`, `ReminderMarkFailedCommitRequest`, `ph1rem_mark_completed`, `ph1rem_mark_failed`.
- Inputs: tenant/user/reminder id, occurrence id, ack source for completion, failure reason for failure, idempotency key.
- Outputs: `REMINDER_COMPLETED` or `REMINDER_FAILED`.
- State changes: reminder and occurrence enter terminal `Completed` or `Failed`.
- Audit evidence: row fields.
- Gaps: no downstream task/calendar completion handoff found.

### Stage 11 — BCAST Handoff / Reminder Fired

- Owner: Selene OS bridge coordinates; PH1.REM owns timing; PH1.BCAST owns broadcast message lifecycle.
- Symbols/files: `run_broadcast_mhp_defer_with_reminder`, `run_broadcast_mhp_mark_reminder_fired`.
- Inputs: BCAST MHP defer request and retry/follow-up time; later fired request.
- Outputs: combined BCAST+REM handoff refs; BCAST reminder-fired result.
- State changes: BCAST recipient state changes to reminder-set or fired state; PH1.REM schedules BCAST follow-up reminder.
- Audit evidence: BCAST/REM proof refs; no PH1.J proof located.
- Gaps: external delivery status still belongs to BCAST/DELIVERY and is partial.

## 5. Data Model / Contracts / Packets

### Request Structs

| Contract / Struct | File | Status | Notes |
| --- | --- | --- | --- |
| `ReminderScheduleCommitRequest` | `crates/selene_kernel_contracts/src/ph1rem.rs` | FOUND | Schedule/create request. |
| `ReminderUpdateFields` | `crates/selene_kernel_contracts/src/ph1rem.rs` | FOUND | Optional update payload. |
| `ReminderUpdateCommitRequest` | `crates/selene_kernel_contracts/src/ph1rem.rs` | FOUND | Update by reminder id. |
| `ReminderCancelCommitRequest` | `crates/selene_kernel_contracts/src/ph1rem.rs` | FOUND | Cancel by reminder id. |
| `ReminderSnoozeCommitRequest` | `crates/selene_kernel_contracts/src/ph1rem.rs` | FOUND | Snooze occurrence. |
| `ReminderFollowupScheduleCommitRequest` | `crates/selene_kernel_contracts/src/ph1rem.rs` | FOUND | Schedule follow-up checkpoint. |
| `ReminderDeliveryRetryScheduleCommitRequest` | `crates/selene_kernel_contracts/src/ph1rem.rs` | FOUND | Schedule retry after failed attempt. |
| `ReminderDeliverPreCommitRequest` | `crates/selene_kernel_contracts/src/ph1rem.rs` | FOUND | Record pre-due delivery attempt. |
| `ReminderDeliverDueCommitRequest` | `crates/selene_kernel_contracts/src/ph1rem.rs` | FOUND | Record due delivery attempt and follow-up/retry. |
| `ReminderEscalateCommitRequest` | `crates/selene_kernel_contracts/src/ph1rem.rs` | FOUND | Escalate from one channel to another. |
| `ReminderMarkCompletedCommitRequest` | `crates/selene_kernel_contracts/src/ph1rem.rs` | FOUND | Mark completed. |
| `ReminderMarkFailedCommitRequest` | `crates/selene_kernel_contracts/src/ph1rem.rs` | FOUND | Mark failed. |
| Natural-language reminder packet | N/A | NOT_FOUND | No PH1.REM-native OpenAI/PH1.N packet found. |

### Response Structs

| Contract / Struct | File | Status | Notes |
| --- | --- | --- | --- |
| `Ph1RemResponse` | `crates/selene_kernel_contracts/src/ph1rem.rs` | FOUND | `Ok` or `Refuse`. |
| `Ph1RemOk` | `crates/selene_kernel_contracts/src/ph1rem.rs` | FOUND | Requires valid ids and no delivery status without delivery attempt id. |
| `Ph1RemRefuse` | `crates/selene_kernel_contracts/src/ph1rem.rs` | FOUND | Simulation id, reason code, message. |

### Records

| Record | File | Status | Notes |
| --- | --- | --- | --- |
| `ReminderRecord` | `crates/selene_storage/src/ph1f.rs` | FOUND | Tenant/user/device, type, text, due time, timezone, recurrence, channels, state, timestamps. |
| `ReminderOccurrenceRecord` | `crates/selene_storage/src/ph1f.rs` | FOUND | Occurrence id, reminder id, scheduled time, state, snooze/follow-up/retry fields, completion/failure fields. |
| `ReminderDeliveryAttemptRecord` | `crates/selene_storage/src/ph1f.rs` | FOUND | Attempt id, reminder id, occurrence id, channel, status, proof ref, created time. |
| SQL table migrations | `crates/selene_storage/migrations` | NOT_FOUND | Docs declare tables; migration was not located. |

### Enums / Status States

| Enum | Values | Status | Notes |
| --- | --- | --- | --- |
| `ReminderType` | `Task`, `Meeting`, `Timer`, `Medical`, `Custom`, `BcastMhpFollowup` | FOUND | Only some variants have active product routes. |
| `ReminderChannel` | `Voice`, `Push`, `Text`, `Email`, `PhoneApp` | FOUND | Channel enum; live providers not proven. |
| `ReminderState` | `Scheduled`, `Snoozed`, `FollowupPending`, `Canceled`, `Completed`, `Failed` | FOUND | No separate delivered state; delivery is attempt status. |
| `ReminderDeliveryStatus` | `Delivered`, `DeferredQuietHours`, `RetryScheduled`, `Failed` | FOUND | `DeferredQuietHours` exists but active quiet-hours branch not located. |
| `ReminderPriorityLevel` | `Low`, `Normal`, `High`, `Critical` | FOUND | Stored; no full emergency policy found. |
| `ReminderLocalTimeMode` | `FixedTimezone`, `LocalTime` | FOUND | Stored; no deep timezone/DST conversion found. |
| `ReminderAckSource` | `Voice`, `Text`, `Ui` | FOUND | Completion ack source. |

### IDs / Tokens

| ID | File | Status | Notes |
| --- | --- | --- | --- |
| `ReminderId` | `crates/selene_kernel_contracts/src/ph1rem.rs` | FOUND | Generated in PH1.F as `reminder_<seq>`. |
| `ReminderOccurrenceId` | `crates/selene_kernel_contracts/src/ph1rem.rs` | FOUND | Generated in PH1.F as `reminder_occurrence_<seq>`. |
| `ReminderDeliveryAttemptId` | `crates/selene_kernel_contracts/src/ph1rem.rs` | FOUND | Generated in PH1.F as `reminder_delivery_attempt_<seq>`. |
| `idempotency_key` | contract request structs | FOUND | Used across schedule/update/delivery-style commits. |

### Audit / Provenance Fields

| Field / Surface | Status | Notes |
| --- | --- | --- |
| `delivery_proof_ref` | FOUND | Stored on delivery attempts and response. |
| `proof_ref` from dispatch outcome | FOUND | App ingress exposes proof token. |
| `created_at`, `updated_at` | FOUND | Records carry timestamps. |
| tenant/user/device/session refs | PARTIAL | Tenant/user/device in records; session refs not present in PH1.REM records. |
| PH1.J audit event | PARTIAL / AUDIT_GAP | Docs declare audit; code-level emission not proven. |

## 6. Reminder Types And Product Functions

| Reminder Type / Product Function | Evidence Found | Current Behavior | Owner | Security Implications | Missing Design |
| --- | --- | --- | --- | --- | --- |
| personal/task reminder | `ReminderType::Task`, SimulationExecutor `SetReminder` | Schedules task reminder after access gate | PH1.REM timing; Access gate upstream | User-scoped reminder mutation requires access decision | Full PH1.D/PH1.N natural-language flow and PH1.WRITE wording boundary. |
| meeting reminder | `ReminderType::Meeting`, `CreateCalendarEvent` route | Creates meeting reminder and returns calendar draft outcome; no external calendar write | PH1.REM timing | Calendar write is avoided; external mutation not performed | Calendar owner integration. |
| timer | `ReminderType::Timer` | Contract enum only | PH1.REM future | Timer may be low risk but still user scoped | Product route not found. |
| medical reminder | `ReminderType::Medical` | Contract enum only | PH1.REM future with health policy owners | Sensitive/high-risk; must not overclaim | Medical safety and privacy policy not implemented. |
| custom reminder | `ReminderType::Custom` | Contract enum only | PH1.REM | Depends on content sensitivity | Product route not found. |
| BCAST follow-up reminder | `ReminderType::BcastMhpFollowup`, OS bridge tests | Active BCAST.MHP defer-to-reminder bridge | PH1.REM timing; BCAST lifecycle | Must not let REM own message content | Live delivery/provider proof partial. |
| onboarding reminder | ONB journey docs, BCAST/REM extraction references | Design/future context; active postpone/resume not found | PH1.ONB plus PH1.REM plus BCAST/DELIVERY | Private onboarding state must be scoped | End-to-end implementation proof missing. |
| link reminder | Link journey docs | Not found as active runtime | PH1.LINK status plus PH1.REM future | Link status/access-sensitive | Direct implementation missing. |
| delivery retry reminder | `ReminderDeliveryRetryScheduleCommitRequest`, `ph1rem_retry` | Retry time stored for reminder delivery attempt | PH1.REM timing | Retry must not spam or bypass provider policy | Provider retry integration missing. |
| emergency escalation reminder | priority `Critical`, escalation commit | Partial escalation and priority support | PH1.REM plus policy/authority owners | High interruption risk | Emergency simulation/policy proof missing. |
| scheduler/roster future reminder | Docs mention future boundaries | No direct integration found | Scheduler/Roster own business state | REM must not mutate roster/schedule | Integration not found. |

## 7. Natural Language / PH1.D / PH1.N / PH1.X Interaction

Current repo truth:

- PH1.REM itself consumes deterministic request structs, not raw human language.
- SimulationExecutor has intent-draft routes for `SetReminder`, `UpdateReminder`, `CancelReminder`, and `CreateCalendarEvent`.
- Required fields for reminder actions include bounded values such as task text, when/time, reminder id, and optional tenant id.
- Access gating is enforced before reminder mutation in SimulationExecutor.
- App ingress has a read-only keyword heuristic for list/show/upcoming reminder queries.
- No direct PH1.D/OpenAI or GPT-5.5 reminder parsing path was found inside PH1.REM.

Correct future rule:

Users may say:

- "remind me tomorrow";
- "come back in 10 minutes";
- "remind Tom after lunch";
- "follow up with Sarah next week";
- "ping him if he does not answer";
- "remind the new employee to finish onboarding tomorrow".

OpenAI/GPT-5.5 through PH1.D may propose meaning.

PH1.N extracts:

- reminder action;
- subject/body;
- time candidate;
- timezone/locale candidate;
- recurrence candidate;
- recipient candidate;
- channel candidate;
- sensitivity/risk candidate;
- missing-field or clarification need.

PH1.X validates lane, owner, risk, required gates, and clarification need.

PH1.REM owns deterministic timing state only.

Repo status:

- Natural-language candidate extraction: PARTIAL via intent drafts outside PH1.REM.
- GPT-5.5 through PH1.D: NOT_FOUND in current PH1.REM runtime.
- Ambiguity refusal: FOUND.
- Human clarification flow: DESIGN_GAP.

## 8. Time, Timezone, Ambiguity, And Recurrence

Current accepted time inputs found in `crates/selene_storage/src/ph1f.rs`:

- `now`;
- `today`;
- any lowercased string containing `tomorrow`;
- relative delay strings of the form `in N sec`, `in N secs`, `in N second`, `in N seconds`, `in N min`, `in N mins`, `in N minute`, `in N minutes`, `in N hour`, `in N hours`, `in N hr`, `in N hrs`, `in N day`, `in N days`;
- numeric timestamp strings interpreted by length as nanoseconds, milliseconds, or seconds.

Current recurrence support:

- `daily`;
- `every day`;
- `weekly`;
- `every week`;
- RRULE-like `FREQ=DAILY;INTERVAL=N;COUNT=N`;
- RRULE-like `FREQ=WEEKLY;INTERVAL=N;COUNT=N`;
- max generated occurrences: 365.

Current missing/partial support:

- Monthly recurrence: NOT_FOUND.
- Rich natural language such as "after lunch" or "next Friday morning": NOT_FOUND in PH1.REM.
- User locale interpretation: PARTIAL; timezone string is stored but not deeply interpreted.
- Daylight savings handling: NOT_FOUND.
- Quiet hours computation: NOT_FOUND in active code; enum/docs exist.
- Clarification question generation: PARTIAL; PH1.REM refuses with `REM_FAIL_TIME_AMBIGUOUS_NEEDS_CONFIRM`, PH1.WRITE flow not wired.
- GPT-5.5 time interpretation: NOT_FOUND.

Answers required by repo truth:

- What time formats are currently accepted? Limited exact, relative, tomorrow/today, and numeric timestamp formats listed above.
- What ambiguous time cases fail closed? Unrecognized desired time text fails with `REM_FAIL_TIME_AMBIGUOUS_NEEDS_CONFIRM`.
- Does PH1.REM ask clarification or only refuse? PH1.REM refuses; upstream must ask.
- Does PH1.WRITE explain ambiguity? Not proven; app ingress has hardcoded failure wording.
- Does GPT-5.5 help with time interpretation now? Not found in current PH1.REM path.
- What needs future upgrade? PH1.D/PH1.N time candidate extraction, PH1.X ambiguity validation, PH1.WRITE clarification, timezone/DST policy, recurrence expansion, quiet hours.

## 9. Reminder Delivery / Notification / Channel Interaction

Current repo truth:

- PH1.REM does not send external messages directly through PH1.BCAST or PH1.DELIVERY.
- PH1.REM records delivery attempts for reminder occurrences.
- `ReminderChannel` supports `Voice`, `Push`, `Text`, `Email`, and `PhoneApp` as enum values.
- `ReminderDeliveryStatus` supports `Delivered`, `DeferredQuietHours`, `RetryScheduled`, and `Failed`.
- Pre-due and due delivery commit requests carry proof refs and create delivery-attempt rows.
- Due delivery may schedule follow-up or retry depending offline/early/snoozed state.
- Boundary script `scripts/check_delivery_ownership_boundaries.sh` verifies the PH1.REM runtime does not directly call PH1.BCAST/PH1.DELIVERY symbols.

Answers required:

- Does PH1.REM send notifications directly? No live external send was found; it records timing/delivery attempt proof.
- Does PH1.REM hand off to PH1.BCAST? BCAST.MHP handoff is coordinated by Selene OS, not direct PH1.REM call.
- Does PH1.REM hand off to PH1.DELIVERY? No direct PH1.DELIVERY handoff found.
- Does PH1.REM only track timing/proof? Current code mostly supports this statement.
- Are channels Voice, Push, Text, Email, PhoneApp implemented? They are contract enum values; provider behavior is not proven.
- Are live provider integrations present? NOT_FOUND.
- Does TTS/voice reminder exist? Voice channel enum exists, but TTS reminder delivery path was not found.
- Does Desktop/iPhone render reminders? PARTIAL via System Activity/Needs Attention surfaces.
- Does Adapter transport reminder actions? PARTIAL via turn/simulation tests; no dedicated route found.

Critical boundary:

PH1.REM owns timing.

PH1.BCAST/PH1.DELIVERY own outbound communication where applicable.

PH1.TTS/PH1.WRITE own spoken/user-facing wording.

Clients render only.

## 10. Interaction With Onboarding

Repo evidence:

- The onboarding journey design requires PH1.REM for postpone/follow-up behavior.
- BCAST/Delivery/Reminder extraction and current docs mention onboarding reminder or ONB backfill timing support.
- `docs/08_SIMULATION_CATALOG.md` includes ONB backfill notification flows that reference broadcast/reminder handoffs.

What was not proven as active runtime:

- A direct PH1.ONB to PH1.REM postpone/resume call path.
- A receiver saying "not now" inside onboarding and PH1.REM scheduling exact remaining-step resume.
- PH1.ONB resuming exact remaining step from a reminder-fired event.

Answers required:

- Can onboarding be postponed? Future design says yes; current active proof not found.
- Does PH1.REM schedule onboarding follow-up? PARTIAL/DESIGN_GAP.
- Does PH1.ONB hand off to PH1.REM? Not proven in code inspection.
- Does BCAST deliver onboarding reminder? Future design/partial, not live-proven.
- Does PH1.ONB resume exact remaining step after reminder? NOT_FOUND as active proof.
- What is active vs design-only? BCAST.MHP follow-up is active; ONB postpone/resume appears design/future.
- What is missing? End-to-end tests, PH1.ONB handoff code, PH1.WRITE wording, delivery proof, JD live proof.

## 11. Interaction With Broadcast / Delivery

Current BCAST.MHP reminder behavior:

- BCAST can defer a recipient and request a reminder handoff.
- Selene OS schedules a PH1.REM `BcastMhpFollowup` reminder with deterministic request text, UTC timezone, PhoneApp channel, and deterministic idempotency key.
- Later, Selene OS can mark the BCAST reminder fired via `BCAST_REMINDER_FIRED_COMMIT`.
- PH1.REM does not call BCAST directly.
- BCAST owns broadcast recipient lifecycle.
- PH1.REM owns reminder timing/occurrence state.

Answers required:

- Does BCAST defer to REM? Yes, through Selene OS bridge for BCAST.MHP.
- Does REM fire and resume BCAST? The OS bridge marks BCAST reminder fired; PH1.REM itself is not direct caller.
- Who owns BCAST reminder state? PH1.BCAST owns broadcast recipient/message lifecycle.
- Who owns reminder occurrence? PH1.REM.
- Who owns delivery retry? PH1.REM owns reminder retry timing; PH1.DELIVERY should own provider-attempt retry where applicable.
- Who owns escalation? PH1.REM has reminder escalation commit; broader policy/authority escalation remains outside.
- Are guardrails present? Yes, boundary script prevents direct BCAST/DELIVERY calls in PH1.REM runtime.

## 12. Interaction With Task / Scheduler / Roster / Calendar

Repo evidence:

- `SetReminder` in SimulationExecutor schedules a `ReminderType::Task` reminder.
- `CreateCalendarEvent` schedules a `ReminderType::Meeting` reminder and returns `CalendarDraftCreated`.
- User-facing response says the calendar draft is created and not sent to an external calendar.
- Access gates are enforced for reminder mutation and calendar draft route.

Not found:

- Direct task engine mutation from PH1.REM.
- Scheduler engine integration.
- Roster engine integration.
- External calendar provider write.
- Meeting schedule canonical owner beyond reminder draft behavior.
- Staff schedule/workload mutation.

Correct future rule:

PH1.REM may provide timing/reminder support, but Task/Scheduler/Roster owners must own their own business state.

Current status:

- Task reminder: PARTIAL.
- Calendar draft reminder: PARTIAL.
- Scheduler/Roster: NOT_FOUND.

## 13. PH1.WRITE / OpenAI / User Guidance Interaction

Current repo truth:

- PH1.REM contract/runtime does not call PH1.WRITE.
- PH1.REM contract/runtime does not call OpenAI/GPT-5.5 or PH1.D.
- App ingress uses deterministic/hardcoded reminder responses:
  - "I scheduled that reminder."
  - "I updated that reminder."
  - "I canceled that reminder."
  - "I couldn't complete that reminder request."
- List reminders path builds a deterministic tool-style response with citations/provenance.

Risk markers:

- `REM_WRITING_OWNER_RISK`: PARTIAL. PH1.REM is not writing directly, but app-ingress hardcoded reminder wording bypasses a fully wired PH1.WRITE boundary.
- `CLIENT_REMINDER_TEXT_RISK`: PARTIAL. Clients appear mostly render-only, but end-to-end PH1.WRITE-owned reminder text is not proven.
- `HARDCODED_REMINDER_GUIDANCE_RISK`: FOUND. App ingress has hardcoded reminder phrasing.

Correct future rule:

OpenAI/GPT-5.5 may help explain, clarify, and phrase reminders through PH1.D.

PH1.WRITE owns final reminder guidance.

PH1.REM must not become the writing brain.

## 14. Desktop / iPhone / Adapter Boundaries

### Desktop

- Evidence: System Activity / Needs Attention render surfaces in `apple/mac_desktop`.
- Status: PARTIAL.
- Current role: display operational state; no local reminder state authority found.
- Risk: `DESKTOP_REM_AUTHORITY_RISK` not proven active, but render-only proof remains needed.

### iPhone

- Evidence: System Activity / Needs Attention surfaces and `broadcast_waiting_followup_reminder_state` in `apple/iphone`.
- Status: PARTIAL.
- Current role: display operational/follow-up state; no local reminder state authority found.
- Risk: `IPHONE_REM_AUTHORITY_RISK` not proven active, but render-only proof remains needed.

### Adapter

- Evidence: adapter tests show calendar event confirmation dispatches simulation and persists meeting reminder; cancel reminder dispatches simulation; list reminders uses read-only tool lane with provenance.
- Status: PARTIAL.
- Current role: transports turn/runtime requests through OS path and exposes read-only listing; no dedicated reminder business/provider route found.
- Risk: `ADAPTER_REM_AUTHORITY_RISK` not proven active, but dedicated transport-only proof remains needed.

### App ingress

- Evidence: `app_ingress.rs` maps simulation outcomes into hardcoded user responses and read-only reminder listing.
- Status: FOUND/PARTIAL.
- Risk: hardcoded wording and keyword listing heuristic need PH1.WRITE/PH1.X reconciliation.

## 15. Security / Privacy / Consent Model

Repo evidence:

- Tenant and user ids are mandatory in reminder requests and records.
- Optional device id is stored and validated when present.
- Schedule requires identity; device id requires existing device proof.
- SimulationExecutor applies access gates for set/update/cancel reminder mutations.
- Non-owner reminder mutation requires approval/access path in SimulationExecutor tests.
- Ownership checks prevent wrong tenant/user mutation inside PH1.REM storage.
- Delivery attempts are append-only guarded.
- Idempotency indexes prevent duplicate schedule/update/delivery effects.
- Terminal state checks prevent invalid mutation after completion/cancel/failure.

Missing or partial:

- Recipient-scope model for reminders to other people: DESIGN_GAP.
- Reminder sharing/visibility policy: DESIGN_GAP.
- Channel consent: DESIGN_GAP.
- External delivery consent: DESIGN_GAP.
- Reminder content classification for private/protected content: DESIGN_GAP.
- Rate limits/caps: DESIGN_GAP.
- PH1.WRITE privacy wording: DESIGN_GAP.
- PH1.J audit emission: AUDIT_GAP.
- Full tenant/workspace policy integration beyond tenant/user ids: PARTIAL.

Security rule extracted:

Raw reminder text does not grant authority.

Reminder timing state does not mutate tasks, rosters, schedules, access, or protected business state.

## 16. Reminder State Machine

Current explicit states:

- `Scheduled`
- `Snoozed`
- `FollowupPending`
- `Canceled`
- `Completed`
- `Failed`

Current explicit delivery statuses:

- `Delivered`
- `DeferredQuietHours`
- `RetryScheduled`
- `Failed`

RECONSTRUCTED_FROM_REPO_EVIDENCE:

```text
Schedule
  -> Scheduled

Update
  Scheduled/Snoozed/FollowupPending -> Scheduled or same active state with updated fields

Snooze
  Scheduled/FollowupPending -> Snoozed

DeliverDue
  Scheduled and due -> FollowupPending + Delivered attempt
  Snoozed before snooze_until -> RetryScheduled delivery status, occurrence remains Snoozed
  Early/offline -> RetryScheduled delivery status

Followup
  Scheduled/Snoozed -> FollowupPending

Retry
  active occurrence -> retry_time set, delivery status RetryScheduled

Escalate
  prior channel attempt + allowed channel -> FollowupPending + Delivered attempt

Cancel
  active reminder -> Canceled

Complete
  active reminder -> Completed

Fail
  active reminder -> Failed
```

Do not claim runtime implements separate states for:

- `Delivered` as reminder state;
- `DeferredQuietHours` as active quiet-hours logic;
- `Expired`;
- `Missed` as explicit state;
- `Blocked` as explicit state.

Those are either delivery statuses, inferred cases, docs concepts, or future gaps.

## 17. Error Handling And Reason Codes

Existing reason codes / failures found:

| Scenario | Current Reason / Behavior | Status | Notes |
| --- | --- | --- | --- |
| ambiguous time | `REM_FAIL_TIME_AMBIGUOUS_NEEDS_CONFIRM` | FOUND | Returned when desired time cannot be resolved. |
| invalid time | same ambiguous time refusal or validation failure | PARTIAL | No rich invalid-time taxonomy found. |
| timezone missing | contract validates required fields | PARTIAL | Timezone stored; missing/empty validation exists, no timezone policy reason found. |
| reminder not found | `REM_FAIL_SCOPE_VIOLATION` / state mutation refusal | FOUND | Used for missing or wrong-scope rows. |
| occurrence not found | `REM_FAIL_SCOPE_VIOLATION` | FOUND | Occurrence must belong to reminder/scope. |
| invalid state transition | `REM_FAIL_STATE_TRANSITION_INVALID` | FOUND | Terminal/invalid mutation guard. |
| scope violation | `REM_FAIL_SCOPE_VIOLATION` | FOUND | Tenant/user ownership mismatch. |
| policy blocked | `REM_FAIL_POLICY_BLOCKED` | FOUND | Used for escalation channel policy and similar blocked paths. |
| quiet hours | enum/docs only | PARTIAL / DESIGN_GAP | No active quiet-hours decision located. |
| channel unavailable | `REM_FAIL_POLICY_BLOCKED` for unsupported escalation channel | PARTIAL | Channel provider availability not present. |
| delivery failed | `ReminderDeliveryStatus::Failed`, mark failed | PARTIAL | Provider failure handling not live-proven. |
| retry exhausted | NOT_FOUND | DESIGN_GAP | No retry exhaustion policy located. |
| already completed | invalid state transition | FOUND | Terminal state guard. |
| already canceled | invalid state transition | FOUND | Terminal state guard. |
| missing simulation | SimulationExecutor has missing simulation fail-closed tests for calendar event | PARTIAL | Not a PH1.REM-internal error. |
| unsupported recurrence | fallback to no recurrence / limited parser | PARTIAL | No rich unsupported recurrence reason found. |
| unsupported channel | contract validates enum; escalation policy blocked if not preferred | PARTIAL | Provider/channel support matrix missing. |
| client route mismatch | NOT_FOUND | DESIGN_GAP | No dedicated client reminder route mismatch taxonomy. |

## 18. Audit / Provenance / Evidence

What is stored:

- reminder row with tenant, user, optional device, state, timestamps;
- occurrence row with scheduled time, state, snooze/follow-up/retry/completion/failure fields;
- delivery attempt row with channel, status, proof ref, created time;
- idempotency indexes for replay resistance;
- app-ingress proof tokens and read-only listing citations.

Required answers:

- Is reminder creation audited? PARTIAL. Rows are persisted in PH1.F; PH1.J audit event not proven.
- Is update audited? PARTIAL. Row changes are stored; explicit audit event not proven.
- Is cancellation audited? PARTIAL. Row changes are stored; explicit audit event not proven.
- Is snooze audited? PARTIAL. Row changes are stored; explicit audit event not proven.
- Is due firing audited? PARTIAL. Delivery attempt row/proof ref exists; explicit audit event not proven.
- Is delivery attempt audited? PARTIAL/FOUND as storage evidence, AUDIT_GAP for PH1.J.
- Is retry audited? PARTIAL. Retry time stored.
- Is escalation audited? PARTIAL. Attempt row and response escalation level exist.
- Is completion/failure audited? PARTIAL. Row state/failure reason stored.
- Are tenant/user/device/session refs recorded? Tenant/user/device yes; session not found in PH1.REM records.
- Are delivery/channel refs recorded? Channel and proof refs yes.
- Are BCAST/ONB/LINK refs recorded where applicable? BCAST.MHP request text and OS bridge refs exist; ONB/LINK direct refs not found.
- Are client/adapter events audited? Adapter/listing provenance exists; explicit audit event not proven.

AUDIT_GAP:

Docs declare PH1.J audit emissions, but current extraction did not locate code-level PH1.J emission for PH1.REM commits.

## 19. Current Tests / Smokes / Acceptance

| Test / Smoke | Path | What It Proves | What It Does Not Prove | Status |
| --- | --- | --- | --- | --- |
| `at_rem_contract_01_schedule_simulation_id_must_match` | `crates/selene_kernel_contracts/src/ph1rem.rs` | Contract rejects simulation-id mismatch | Runtime behavior, delivery, UI | FOUND |
| `at_rem_contract_02_ok_delivery_requires_attempt_id` | `crates/selene_kernel_contracts/src/ph1rem.rs` | Response validation prevents delivery status without attempt id | Provider delivery | FOUND |
| `at_rem_01_schedule_idempotent` | `crates/selene_os/src/ph1rem.rs` | Runtime schedule idempotency | Live provider, UI | FOUND |
| `at_rem_02_unparseable_time_refused` | `crates/selene_os/src/ph1rem.rs` | Ambiguous/unparseable time refuses | Clarification UX | FOUND |
| `at_rem_db_01_schedule_round_trip_reads_same_rows` | `crates/selene_os/src/ph1rem.rs` | Storage round-trip rows | SQL migration proof | FOUND |
| `at_rem_db_02_delivery_attempts_append_only_guard` | `crates/selene_os/src/ph1rem.rs` | Append-only delivery attempt guard | Provider delivery | FOUND |
| `at_rem_db_03_delivery_replay_safe_after_index_rebuild` | `crates/selene_os/src/ph1rem.rs` | Replay safety after index rebuild | Distributed persistence | FOUND |
| `at_rem_db_04_schedule_with_recurrence_expands_occurrences` | `crates/selene_os/src/ph1rem.rs` | Recurrence occurrence expansion | Monthly/complex recurrence | FOUND |
| `at_rem_db_05_update_rebuilds_occurrences_for_recurrence_change` | `crates/selene_os/src/ph1rem.rs` | Update rebuilds occurrences | User-facing confirmation | FOUND |
| `at_rem_db_06_cancel_marks_all_generated_occurrences_canceled` | `crates/selene_os/src/ph1rem.rs` | Cancel cascades to generated occurrences | Delivery cancellation provider | FOUND |
| `at_rem_db_07_due_fires_once` | `crates/selene_os/src/ph1rem.rs` | Due delivery is idempotent | Live notification | FOUND |
| `at_rem_db_08_retry_does_not_double_send` | `crates/selene_os/src/ph1rem.rs` | Retry avoids duplicate delivered attempts | Provider retry policy | FOUND |
| `at_rem_db_09_escalate_requires_policy` | `crates/selene_os/src/ph1rem.rs` | Escalation requires channel policy conditions | Emergency escalation policy | FOUND |
| `at_rem_db_10_snooze_blocks_due_until_snooze_window_ends` | `crates/selene_os/src/ph1rem.rs` | Snooze suppresses due firing until window | Quiet hours | FOUND |
| `at_rem_db_11_missed_due_delivery_sets_followup_pending` | `crates/selene_os/src/ph1rem.rs` | Missed due delivery tightens follow-up | External follow-up delivery | FOUND |
| `at_sim_exec_01a_set_reminder_routes_to_ph1rem_schedule_commit` | `crates/selene_os/src/simulation_executor.rs` | Intent route to PH1.REM schedule | GPT/NLP understanding | FOUND |
| `at_sim_exec_update_reminder_routes_to_ph1rem_update_commit` | `crates/selene_os/src/simulation_executor.rs` | Intent route to update | Rich update UX | FOUND |
| `at_sim_exec_cancel_reminder_routes_to_ph1rem_cancel_commit` | `crates/selene_os/src/simulation_executor.rs` | Intent route to cancel | UI confirmation quality | FOUND |
| `at_sim_exec_set_reminder_access_scope_violation_when_action_not_allowed` | `crates/selene_os/src/simulation_executor.rs` | Access gate denies reminder mutation | Full policy matrix | FOUND |
| `at_sim_exec_set_reminder_non_owner_requires_ap` | `crates/selene_os/src/simulation_executor.rs` | Non-owner mutation requires AP | All authority flows | FOUND |
| `at_sim_exec_01b_bcast_mhp_defer_hands_off_to_rem_and_returns_handoff_refs` | `crates/selene_os/src/simulation_executor.rs` | BCAST.MHP defer schedules PH1.REM | Live BCAST delivery | FOUND |
| `at_bcast_mhp_05_reminder_set_and_fired_flow_via_ph1_rem` | `crates/selene_os/src/simulation_executor.rs` and script | BCAST reminder-fired flow via PH1.REM | External notification provider | FOUND |
| `at_sim_exec_calendar_event_schedules_meeting_reminder_and_is_idempotent` | `crates/selene_os/src/simulation_executor.rs` | Calendar draft creates meeting reminder | External calendar write | FOUND |
| `at_sim_exec_calendar_event_non_owner_role_requires_ap` | `crates/selene_os/src/simulation_executor.rs` | Calendar/reminder non-owner access gate | Full calendar governance | FOUND |
| `at_sim_exec_calendar_event_missing_sim_registration_fails_closed` | `crates/selene_os/src/simulation_executor.rs` | Missing sim registration fails closed | All reminder sim registry paths | FOUND |
| `at_adapter_03ba_calendar_event_confirm_yes_dispatches_sim_and_persists_meeting_reminder` | `crates/selene_adapter/src/lib.rs` | Adapter confirmation path can persist meeting reminder | Dedicated route or live UI | FOUND |
| `at_adapter_03bb_cancel_reminder_confirm_yes_dispatches_sim_and_cancels_row` | `crates/selene_adapter/src/lib.rs` | Adapter can dispatch cancel after confirmation | All cancel UX states | FOUND |
| `at_adapter_03bc_list_reminders_uses_read_only_tool_lane_with_provenance` | `crates/selene_adapter/src/lib.rs` | Reminder listing is read-only with provenance | PH1.WRITE answer boundary | FOUND |
| `scripts/check_delivery_ownership_boundaries.sh` | `scripts/check_delivery_ownership_boundaries.sh` | PH1.REM runtime does not directly call BCAST/DELIVERY | Dynamic runtime call graph | FOUND |
| `scripts/check_bcast_mhp_acceptance.sh` | `scripts/check_bcast_mhp_acceptance.sh` | Acceptance script requires BCAST MHP reminder flow | Wider provider proof | FOUND |

TEST_GAP:

- Natural-language time interpretation via PH1.D/PH1.N.
- PH1.WRITE reminder wording.
- Live SMS/email/push/voice provider reminders.
- Onboarding postpone/resume reminder.
- Desktop/iPhone render-only proof.
- Adapter dedicated transport-only proof.
- SQL persistence/migration proof.
- JD live acceptance.

## 20. Old Paths / Compatibility / Wrong-Owner Risks

| Path / Symbol | Current Status | Correct Canonical Owner | Retirement Condition | Active-Caller Check Needed |
| --- | --- | --- | --- | --- |
| BCAST.MHP reminder handoff | Active OS bridge | PH1.REM timing, PH1.BCAST lifecycle, OS orchestration | Retire only if replaced by canonical event bus with equivalent tests | Verify only Selene OS can bridge. |
| App-ingress reminder wording | Active hardcoded wording | PH1.WRITE final wording | Replace after PH1.WRITE reminder presentation proof | Check all user-facing reminder strings. |
| App-ingress list reminder keyword heuristic | Active read-only shortcut | PH1.X/PH1.N future query routing; PH1.REM read-only status | Retire after canonical status assistant exists | Ensure no mutation happens through listing path. |
| PH1.REM delivery attempt records | Active | PH1.REM timing proof; PH1.DELIVERY provider truth | Keep as internal proof; integrate provider delivery separately | Ensure no live provider send is hidden here. |
| `ReminderChannel` enum as provider implication | Contract only | PH1.DELIVERY/provider governance for live send | Live provider activation and policy proof | Do not overclaim provider support. |
| `DeferredQuietHours` delivery status | Enum/doc only | PH1.REM plus quiet-hours policy owner | Implement with tests or mark unsupported | Prevent user-facing claims of quiet-hours behavior. |
| Calendar event route creating meeting reminder | Active draft behavior | Calendar owner future; PH1.REM timing only | Calendar integration proof | Ensure no external calendar write occurs silently. |
| Onboarding reminder future path | Design/future | PH1.ONB plus PH1.REM/BCAST/DELIVERY | ONB postpone/resume proof | Ensure PH1.ONB does not own timing. |
| Client System Activity rendering | Partial | Desktop/iPhone render-only | Render-only proof | Ensure no client state mutation authority. |
| Adapter reminder dispatch/listing | Partial | Adapter transport only | Dedicated transport proof | Ensure Adapter does not own reminder decisions. |
| Docs declaring SQL tables without migration | Stale/partial | Storage owner | Migration/persistence activation pack | Do not claim SQL persistence before proof. |

## 21. Full Functionality Master List

| Functionality | Description | Evidence | Owner | Status | Future Action |
| --- | --- | --- | --- | --- | --- |
| schedule reminder | Create reminder and occurrence(s) from deterministic request | `ph1rem_schedule` | PH1.REM | FOUND | Expand PH1.D/PH1.N input path and PH1.WRITE UX. |
| update reminder | Update text/time/priority/recurrence/channels | `ph1rem_update` | PH1.REM | FOUND | Add review/confirmation UX. |
| cancel reminder | Cancel reminder and active occurrences | `ph1rem_cancel` | PH1.REM | FOUND | Add user-facing proof/status. |
| snooze reminder | Snooze occurrence until future timestamp | `ph1rem_snooze` | PH1.REM | FOUND | Add natural-language snooze interpretation. |
| create recurring reminders | Generate bounded daily/weekly occurrences | `parse_reminder_recurrence_plan` | PH1.REM | PARTIAL | Add richer recurrence and policy. |
| generate occurrences | Create occurrence records for recurrence | `generate_reminder_occurrences` | PH1.REM | FOUND | Add persistence/migration proof. |
| deliver pre reminder | Append pre-due delivery attempt | `ph1rem_deliver_pre` | PH1.REM proof; Delivery future | PARTIAL | Wire provider-governed delivery if approved. |
| deliver due reminder | Append due delivery attempt, retry, or follow-up | `ph1rem_deliver_due` | PH1.REM proof; Delivery future | PARTIAL | Clarify provider/channel owner. |
| schedule retry | Store retry time for occurrence | `ph1rem_retry` | PH1.REM timing | FOUND | Add retry exhaustion and delivery owner integration. |
| schedule follow-up | Store follow-up pending state/time | `ph1rem_followup` | PH1.REM timing | FOUND | Add follow-up status assistant. |
| escalate reminder | Escalate from one channel to an allowed channel | `ph1rem_escalate` | PH1.REM + policy owner | PARTIAL | Add emergency/abuse policy. |
| mark completed | Mark occurrence/reminder completed | `ph1rem_mark_completed` | PH1.REM | FOUND | Add task/calendar handoff if needed. |
| mark failed | Mark occurrence/reminder failed | `ph1rem_mark_failed` | PH1.REM | FOUND | Add failure UX and retry options. |
| idempotency | Prevent duplicate schedule/update/delivery effects | PH1.F indexes | PH1.REM | FOUND | Add distributed persistence proof. |
| append-only delivery attempt guard | Prevent overwrite of delivery attempts | tests/storage guard | PH1.REM | FOUND | Preserve in future SQL implementation. |
| BCAST reminder-fired resume | Coordinate BCAST.MHP follow-up via OS bridge | SimulationExecutor tests | OS + PH1.BCAST + PH1.REM | FOUND | Reconcile into canonical event architecture. |
| onboarding reminder | Future postpone/follow-up support | ONB docs/design | PH1.ONB + PH1.REM | PARTIAL | Implement only with ONB proof. |
| list reminders | Read-only list of user reminders with provenance | `maybe_build_list_reminders_tool_response` | PH1.REM read model + app ingress | FOUND / PARTIAL | Move through status assistant and PH1.WRITE. |
| calendar draft reminder | Create meeting reminder for calendar draft | SimulationExecutor | PH1.REM timing | PARTIAL | Add calendar owner integration. |
| quiet-hours deferral | Delivery status enum/docs | PH1.REM + policy future | NOT_FOUND runtime | Implement or do not claim. |
| live notification provider | SMS/email/push/voice send | None found | PH1.DELIVERY/provider governance | NOT_FOUND | Provider-off/fake/live proofs required. |

## 22. Comparison To Master Architecture

### Global Request Decision Lattice

Current reminder mutations route through SimulationExecutor and access checks, but full PH1.X reminder classification and ambiguity clarification architecture is not fully wired. Future PH1.X must classify reminder creation/update/cancel/list/follow-up by action effect, data scope, recipient, time ambiguity, channel risk, protected content, and confirmation need.

### PH1.D Proposal Gateway

No direct PH1.D/OpenAI reminder path was found. Future PH1.D may propose time interpretation, recurrence normalization, wording, and troubleshooting suggestions, but PH1.REM must remain deterministic timing owner.

### PH1.N Meaning Unravelling

Current intent drafts imply upstream meaning extraction, but PH1.REM does not own semantic parsing. Future PH1.N should extract candidate time, recurrence, recipient, channel, action, and ambiguity evidence.

### PH1.WRITE Human Presentation

Current hardcoded reminder responses in app ingress are not the full PH1.WRITE boundary. Future PH1.WRITE must own reminder confirmations, ambiguity prompts, status summaries, failure explanations, and TTS-safe reminder guidance.

### PH1.BCAST / PH1.DELIVERY

Repo truth correctly separates PH1.REM timing from BCAST/DELIVERY communication. BCAST.MHP handoff exists through Selene OS. Live channel provider delivery remains missing.

### PH1.ONB Onboarding Journey

Onboarding journey design requires reminders for postponement/follow-up. Active ONB-to-REM exact resume proof was not found. PH1.ONB must own onboarding state and PH1.REM must own timing only.

### PH1.LINK Link Journey

Link reminders/status follow-up are future architecture; no direct PH1.LINK-to-PH1.REM runtime path was located. PH1.LINK owns link lifecycle, PH1.REM may later time follow-ups.

### Broadcast Advanced Delivery Modes

Advanced delivery mode, privacy negotiation, quiet hours, acknowledgement, emergency escalation, and view confirmation are future concerns. PH1.REM currently has priority/channel/status fields but not the full advanced delivery architecture.

### Task/Scheduler/Roster Future Stacks

PH1.REM can create timing records and calendar drafts, but no task/scheduler/roster mutation is proven. Future reconciliation must keep those owners separate.

### Identity + Access + Authority Spine

SimulationExecutor enforces access gates for reminder mutation. PH1.REM itself validates tenant/user/device scope. Authority/protected execution remains outside PH1.REM.

### Desktop/iPhone Render-Only Boundary

Clients show operational surfaces. Render-only proof remains needed before claiming fully safe client boundary.

### Adapter Transport-Only Boundary

Adapter tests prove transport/listing behavior. Dedicated transport-only proof remains needed for all reminder actions.

### Old Compatibility Path Retirement

Hardcoded app-ingress wording, keyword list heuristic, SQL-doc/migration mismatch, and BCAST.MHP handoff should remain until canonical replacements have tests and active-caller proof.

## 23. Gaps / Missing Pieces

| Gap | Evidence | Risk | Recommended Future Action | Priority |
| --- | --- | --- | --- | --- |
| missing natural-language reminder understanding | PH1.REM consumes structs; no GPT path found | Users cannot safely say messy reminder requests end to end | Build PH1.D + PH1.N reminder proposal shell | High |
| missing PH1.WRITE reminder wording boundary | App ingress hardcoded responses | Robotic or unsafe user-facing wording | Route confirmations/status/failure through PH1.WRITE | High |
| missing OpenAI/PH1.D proposal path | No PH1.D call in PH1.REM | Provider governance absent for reminder understanding | Add provider-off/fake-provider proof before live | High |
| missing ambiguous time clarification flow | PH1.REM refuses only | User must recover manually | Add PH1.WRITE clarification loop after PH1.REM refusal | High |
| missing timezone/locale policy | Timezone stored but not deeply interpreted | Wrong reminders across locales/DST | Build timezone policy and tests | High |
| missing recurrence coverage | Daily/weekly only | User expectations for monthly/custom recurrence fail | Expand recurrence intentionally with tests | Medium |
| missing reminder delivery proof | Delivery attempt proof only | User may believe SMS/push/voice sent when not live | Wire BCAST/DELIVERY provider proof or label local proof only | High |
| missing live notification provider | No live provider route found | False live-send claims | Provider-off/fake/live activation pack | High |
| missing onboarding postpone/resume proof | ONB design only | Incomplete onboarding reminders may lose context | Build ONB/REM/BCAST handoff proof | High |
| missing status assistant | Read-only list heuristic only | Poor troubleshooting/status UX | Build PH1.REM status assistant through PH1.WRITE | Medium |
| missing scheduler/task integration | Calendar draft only, no task/roster integration | Wrong-owner mutations if improvised later | Create boundary map before implementation | High |
| missing audit | PH1.F rows, no PH1.J emission found | Weak compliance evidence | Add audit evidence pack | High |
| missing SQL persistence | No migration found | In-memory/runtime-only risk | Add migration/persistence activation pack | High |
| missing client render-only proof | UI surfaces partial | Client authority drift risk | Desktop/iPhone render-only reminder proof | Medium |
| missing Adapter transport-only proof | Adapter tests partial | Adapter could become hidden owner later | Adapter transport-only proof | Medium |
| missing JD live acceptance | No live acceptance artifact | Experience unverified | JD live reminder acceptance pack | Medium |
| quiet hours not active | enum/docs only | Overclaiming interruption-aware behavior | Implement or mark unsupported in UI | Medium |
| medical reminder safety undefined | enum only | High-risk health claims | Add safety policy before product activation | High |

## 24. Recommended Future Build Slices

Based on repo truth, future build slices should be:

1. PH1.REM Repo-Truth Activation Pack
2. Reminder Contract / State Machine Normalization
3. Natural-Language Reminder Understanding / PH1.D + PH1.N
4. Time / Timezone / Ambiguity Clarification
5. PH1.WRITE Reminder Guidance Boundary
6. Reminder Schedule / Update / Cancel Proof
7. Snooze / Follow-Up / Retry Proof
8. BCAST Reminder-Fired Handoff Proof
9. Onboarding Postpone / Resume Reminder Proof
10. Delivery Channel / Provider Proof
11. Reminder Status Assistant
12. Task/Scheduler/Roster Boundary Map
13. SQL Persistence / Migration Proof
14. Reminder Audit Evidence Pack
15. Desktop/iPhone Render-Only Reminder Proof
16. Adapter Transport-Only Reminder Proof
17. Quiet Hours / Interruption Policy Proof
18. Recurrence Expansion Proof
19. JD Live Reminder Acceptance Pack

Do not implement from this extraction document alone.

## 25. What Codex Must Not Do

Codex must not:

- invent reminder behavior;
- create duplicate reminder engine;
- let BCAST own timing;
- let DELIVERY own timing;
- let ONB own timing;
- let Desktop/iPhone decide reminder state;
- let Adapter decide reminder state;
- let PH1.REM send messages directly unless repo truth explicitly proves a canonical delivery owner path;
- mutate tasks/schedules/rosters from PH1.REM alone;
- bypass PH1.WRITE for user-facing reminder guidance where unsafe;
- use GPT-5.5 to execute reminders directly;
- claim live SMS/email/push/voice reminder support from enum values or proof refs alone;
- claim quiet-hours support from an enum without active policy proof;
- claim SQL persistence from DB docs without migration proof;
- treat medical reminders as safe product behavior without explicit safety policy;
- delete old paths before proof;
- implement from this extraction document alone.

## 26. Final Extracted Architecture Sentence

PH1.REM is Selene's governed reminder timing boundary: it may schedule, update, snooze, retry, fire, escalate, complete, or fail reminder occurrences where repo truth supports it, but communication delivery, message wording, task/scheduler/roster mutation, onboarding state, access, authority, and protected execution must remain owned by their canonical Selene engines.
