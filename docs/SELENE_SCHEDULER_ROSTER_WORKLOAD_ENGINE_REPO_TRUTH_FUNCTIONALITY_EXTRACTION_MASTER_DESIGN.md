# Selene Scheduler / Roster / Workload Engine — Repo-Truth Functionality Extraction Master Design

REPO_TRUTH_EXTRACTION
NOT_RUNTIME_IMPLEMENTATION
PENDING_GRAND_ARCHITECTURE_RECONCILIATION

## 0. Authority and Scope

AGENTS.md controls execution.

This is docs-only repo-truth extraction.

No runtime code was changed.

This document does not authorize implementation.

The document reconstructs current Scheduler / Roster / Workload / Shift / Availability / Leave / Attendance / Timesheet design and functionality from repo evidence.

Future implementation/refactor/retirement requires explicit build instruction, approved file scope, tests, backend evidence, JD approval, and where visible JD live proof.

This extraction does not claim that the current repo already contains a complete workforce Scheduler/Roster/Workload engine. It separates current repo truth from partial surfaces, future architecture intent, and missing areas. Missing or underdefined areas are marked NOT_FOUND, PARTIAL, UNKNOWN, REPO_TRUTH_NEEDED, DESIGN_GAP, TEST_GAP, OWNER_GAP, AUDIT_GAP, SECURITY_GAP, or COMPLIANCE_GAP.

## 1. Executive Summary

The repo contains an active engine named `PH1.SCHED`, but repo evidence shows it is not a workforce scheduling, roster, shift, attendance, timesheet, leave, or workload allocation engine. Current `PH1.SCHED` is an OS-internal deterministic retry/wait/fail scheduler for WorkOrder orchestration. Its contract is `SCHED_POLICY_EVALUATE -> SCHED_DECISION_COMPUTE`, and its outputs are `RETRY_AT`, `FAIL`, or `WAIT`.

There is no standalone workforce `Scheduler` engine found under `crates/selene_kernel_contracts/src`, `crates/selene_engines/src`, or `crates/selene_os/src`. No `ph1roster`, `ph1workload`, `ph1calendar`, `ph1shift`, `ph1attendance`, `ph1timesheet`, `ph1leave`, or `ph1availability` runtime files were found. No workforce roster, shift, leave, attendance, timesheet, or contractor-hours migration was found.

There is no standalone Roster engine found. Roster appears in future-oriented docs and in adapter protected-shadow heuristics, but no canonical roster state machine, roster table, roster publish flow, or shift assignment flow is implemented in current repo truth.

There is no standalone Workload engine found for human work allocation. The repo contains `PH1.WORK`, but repo evidence shows `PH1.WORK` owns WorkOrder ledger decisions for deterministic orchestration, not workforce workload, capacity planning, staff allocation, contractor progress, or shift workload.

Calendar-adjacent functionality exists only in a bounded form. `PH1.N` can normalize `IntentType::CreateCalendarEvent`, `PH1.X` can ask for confirmation, and `SimulationExecutor` handles confirmed calendar-event intent by creating a PH1.REM `ReminderType::Meeting` record after an access gate. The app/adapter response is explicit: `Draft created; not sent to external calendar yet.` This is PARTIAL calendar-draft behavior, not a live external calendar scheduler and not workforce scheduling.

Position integration exists only as a reference-level surface. `PH1.POSITION` has `PositionScheduleType::{FullTime, PartTime, Contract, Shift}` and a `PositionRecord.schedule_type`, but the PH1.POSITION repo-truth extraction explicitly marks roster group, schedule group, availability, shift pattern, and work-location registry as missing. Position schedule type is not schedule truth.

Onboarding integration is design-forward but not runtime Scheduler/Roster mutation. The onboarding journey design says onboarding may collect start date, location, role, roster group, and availability and may pass approved fields to Scheduler/Roster only through the canonical owner and simulation. It also states no roster/schedule mutation from PH1.ONB alone. Repo-truth runtime evidence for a Scheduler/Roster owner receiving such a handoff was not found.

Access integration is design-forward but not implemented as a live off-work posture. Master Access journey docs define leave/off-shift posture as a future access input from Scheduler/Roster or HR/Payroll. Runtime evidence for Scheduler/Roster notifying Access of leave, sick, off-shift, roster, or shift status was not found.

Reminder integration is partial only for reminder timing and calendar drafts. PH1.REM owns reminder timing and can create a meeting reminder for a calendar draft, but PH1.REM does not own schedule truth, roster truth, shift truth, attendance truth, or task/workload state.

Broadcast/delivery integration is not proven for roster notifications. PH1.BCAST / PH1.DELIVERY docs define future task/scheduler/roster handoff boundaries, but repo truth does not prove roster publish notifications, shift change notifications, leave approval messages, or live provider delivery for workforce scheduling.

Current active surfaces:

- `PH1.SCHED` deterministic WorkOrder retry/wait/fail scheduler.
- `PH1.WORK` WorkOrder ledger and WorkOrder current-state orchestration support.
- `PH1.N` calendar-event intent normalization.
- `PH1.X` calendar-event confirmation and simulation candidate dispatch.
- `SimulationExecutor` calendar-draft path that schedules a PH1.REM meeting reminder.
- `PH1.POSITION` schedule type reference.
- adapter protected-shadow heuristics for phrases such as roster and leave approval.

Current partial surfaces:

- calendar draft is reminder-backed and explicitly not external calendar delivery.
- position schedule type is a job attribute, not roster allocation.
- future docs describe Scheduler/Roster handoffs, but runtime owner evidence is missing.
- app/adapter clients show no workforce roster UI or authority surface; only unrelated native timing/session/wake scheduling strings were found.

Current missing surfaces:

- workforce schedule creation/update/cancellation.
- roster creation/publish/update.
- roster group and schedule group truth.
- shift creation/assignment/change/swap/acceptance/completion.
- employee availability.
- leave request/approval/sick/off-shift truth.
- clock-in/clock-out, breaks, attendance, timesheets.
- contractor time, estimated hours, overrun tracking, manager approval.
- overtime, public holiday, labor compliance, timezone/DST policy for work schedules.
- payroll timesheet handoff.
- finance/budget staffing-cost handoff.
- roster notifications and shift reminders.
- client render-only proof for workforce scheduling.
- adapter transport-only proof for workforce scheduling.
- JD live Scheduler/Roster acceptance proof.

The biggest risk is the name collision: `PH1.SCHED` is already real and authoritative, but it is not the future human-facing Scheduler/Roster/Workload stack described by Position, Onboarding, Access, Reminder, and Broadcast journey docs. Future architecture must reconcile the existing `PH1.SCHED` operational scheduler with any workforce Scheduler/Roster owner before implementation.

## 2. Repo Evidence Inventory

| Evidence Area | File / Path | Symbols / Routes / Tables | Status | Notes |
| --- | --- | --- | --- | --- |
| controlling law | `AGENTS.md` | No Python; shell-only inspection; work only in repo containing AGENTS.md | FOUND | Controls this docs-only extraction. |
| engine inventory | `docs/SELENE_AUTHORITATIVE_ENGINE_INVENTORY.md` | `PH1.SCHED` row | FOUND | Defines PH1.SCHED as deterministic retry/wait/fail decision engine, not workforce roster. |
| PH1.SCHED kernel contract | `crates/selene_kernel_contracts/src/ph1sched.rs` | `SchedCapabilityId`, `SchedDecisionAction`, `SchedRequestEnvelope`, `SchedPolicyEvaluateRequest`, `SchedPolicyEvaluateOk`, `SchedDecisionComputeRequest`, `SchedDecisionComputeOk`, `SchedRefuse`, `Ph1SchedRequest`, `Ph1SchedResponse` | FOUND | Operational WorkOrder retry scheduler. |
| PH1.SCHED engine | `crates/selene_engines/src/ph1sched.rs` | `Ph1SchedRuntime`, `Ph1SchedConfig`, `SCHED_RETRY_SCHEDULED`, `SCHED_MAX_RETRIES_REACHED`, `SCHED_TIMEOUT`, `SCHED_NOT_RETRYABLE` | FOUND | Computes deterministic retry/wait/fail posture. |
| PH1.SCHED OS wiring | `crates/selene_os/src/ph1sched.rs` | `Ph1SchedWiring`, `SchedTurnInput`, `SchedForwardBundle`, `SchedWiringOutcome` | FOUND | OS invokes and validates scheduler decisions; disabled path exists. |
| PH1.SCHED docs | `docs/DB_WIRING/PH1_SCHED.md` | Tables owned: NONE; emits `RETRY_AT | FAIL | WAIT` | FOUND | Explicitly no direct writes in current runtime slice. |
| PH1.SCHED ECM | `docs/ECM/PH1_SCHED.md` | `SCHED_POLICY_EVALUATE`, `SCHED_DECISION_COMPUTE` | FOUND | Allowed caller is Selene OS only; side effects none. |
| core table boundary | `docs/DB_WIRING/SELENE_OS_CORE_TABLES.md`, `docs/ECM/SELENE_OS_CORE_TABLES.md` | WorkOrder orchestration consumes PH1.SCHED decisions | FOUND | Scheduler outputs reflected through WorkOrder ledger rules. |
| simulation catalog | `docs/08_SIMULATION_CATALOG.md` | `SCHED_NEXT_ACTION_DRAFT`, owning domain `Scheduler` | FOUND | Domain label is Scheduler, but content is deterministic retry/timeout next action. |
| coverage matrix | `docs/COVERAGE_MATRIX.md` | `PH1.SCHED` done; `SCHED_POLICY_EVALUATE -> SCHED_DECISION_COMPUTE` | FOUND | Locked for operational scheduler only. |
| WorkOrder ledger | `crates/selene_kernel_contracts/src/ph1work.rs` | `WorkOrderId`, `WorkOrderStatus`, `WorkOrderLedgerEventInput`, `WorkOrderLedgerEvent` | FOUND | Not workforce workload. |
| WorkOrder docs | `docs/DB_WIRING/PH1_WORK.md`, `docs/ECM/PH1_WORK.md` | WorkOrder append/no-op decision boundary | FOUND | PH1.WORK is orchestration ledger, not staff workload. |
| WorkOrder tables | `docs/04_KERNEL_CONTRACTS.md` | `work_order_ledger`, `work_orders_current`, `work_order_step_attempts`, `work_order_leases` | FOUND | Includes retry fields such as `next_retry_at`; not shift/timekeeping. |
| position schedule reference | `crates/selene_kernel_contracts/src/ph1position.rs` | `PositionScheduleType::{FullTime, PartTime, Contract, Shift}`, `PositionRecord.schedule_type` | PARTIAL | Job attribute only; no actual shifts, rosters, or availability. |
| position storage/repo | `crates/selene_storage/src/ph1f.rs`, `crates/selene_storage/src/repo.rs` | `schedule_type: PositionScheduleType` | PARTIAL | Persistence path for position schedule type, not scheduler truth. |
| onboarding schedule docs | `docs/SELENE_PH1ONB_ONBOARDING_JOURNEY_INTELLIGENCE_GUIDED_ENROLLMENT_MASTER_DESIGN.md` | roster/schedule group, availability, no roster/schedule mutation from ONB alone | PARTIAL | Future handoff design; no runtime Scheduler/Roster owner found. |
| position extraction | `docs/SELENE_PH1POSITION_POSITION_ENGINE_REPO_TRUTH_FUNCTIONALITY_EXTRACTION_MASTER_DESIGN.md` | roster group, availability, schedule group marked missing | FOUND | Confirms Position schedule type is not roster truth. |
| position journey | `docs/SELENE_PH1POSITION_POSITION_JOURNEY_INTELLIGENCE_ACCESS_COMPENSATION_HANDOFF_MASTER_DESIGN.md` | Scheduler/Roster owns shifts, roster groups, workload, leave/off-shift truth | PARTIAL | Future design, not current runtime. |
| PH1.N calendar event | `crates/selene_kernel_contracts/src/ph1n.rs`, `crates/selene_engines/src/ph1n.rs` | `IntentType::CreateCalendarEvent`, `normalize_calendar_event` | PARTIAL | Calendar intent normalization exists. |
| PH1.X calendar confirmation | `crates/selene_os/src/ph1x.rs` | `confirm_text` for `CreateCalendarEvent`; `at_x_confirm_yes_dispatches_simulation_candidate_for_calendar_event` | PARTIAL | Confirmation and simulation candidate dispatch exist. |
| calendar draft execution | `crates/selene_os/src/simulation_executor.rs` | `SimulationDispatchOutcome::CalendarDraftCreated`, `enforce_calendar_event_create_gate`, `CALENDAR_EVENT_CREATE`, `ReminderType::Meeting` | PARTIAL | Creates meeting reminder only; no external calendar send. |
| app calendar response | `crates/selene_os/src/app_ingress.rs` | `CALENDAR_DRAFT_CREATED`, response `Draft created; not sent to external calendar yet.` | PARTIAL | User-facing truth is draft-only. |
| adapter calendar path | `crates/selene_adapter/src/lib.rs` | `seed_calendar_access_instance`, `at_adapter_03ba_calendar_event_confirm_yes_dispatches_sim_and_persists_meeting_reminder` | PARTIAL | Adapter can drive calendar draft test; not a workforce scheduler route. |
| adapter protected-shadow heuristics | `crates/selene_adapter/src/lib.rs` | `h380_detects_protected_intent`, `h380_simulation_candidate` with roster/leave strings | PARTIAL | Detects protected roster/leave shadows; does not execute roster/leave. |
| storage migrations | `crates/selene_storage/migrations` | no roster/shift/attendance/timesheet/leave/calendar workforce tables found | NOT_FOUND | Existing migrations cover WorkOrder, access, position schema, reminders, memory, wake, etc. |
| client surfaces | `apple/mac_desktop`, `apple/iphone` | no workforce scheduler/roster UI found; wake/session/local availability scheduling strings only | NOT_FOUND | Native schedule strings are timers/wake/session, not roster. |
| web search parallel scheduler | `crates/selene_os/src/web_search_plan/parallel/scheduler.rs` | parallel web-search scheduler | NOT_TARGET | Search-task scheduling helper, not workforce Scheduler/Roster. |
| tests | multiple Rust tests | PH1.SCHED tests; calendar draft tests; no roster/shift/leave/timesheet tests found | PARTIAL | Tests prove operational scheduler and calendar draft, not workforce Scheduler/Roster. |

## 3. Current Owner Map

| Responsibility | Current Owner / File | Correct Future Owner Hypothesis | Status | Notes |
| --- | --- | --- | --- | --- |
| schedule creation | `PH1.SCHED` only for WorkOrder retry decisions; `SimulationExecutor` for calendar draft reminder | Workforce Scheduler/Calendar owner | PARTIAL | No work schedule creation found. Calendar draft creates a meeting reminder only. |
| schedule update | NOT_FOUND | Workforce Scheduler/Calendar owner | NOT_FOUND | PH1.SCHED has no human schedule update. |
| schedule cancellation | PH1.REM cancel exists for reminders, not schedules | Workforce Scheduler/Calendar owner | NOT_FOUND | No schedule cancel owner found. |
| roster creation | NOT_FOUND | Roster owner | NOT_FOUND | No roster draft/storage/publish model found. |
| roster publish | NOT_FOUND | Roster owner + BCAST/DELIVERY notifications | NOT_FOUND | No publish state or notification proof. |
| roster update | NOT_FOUND | Roster owner | NOT_FOUND | Adapter strings only detect roster risk. |
| roster group | Future ONB/Position docs only | Roster owner | PARTIAL | No canonical roster group record found. |
| shift creation | NOT_FOUND | Shift/Roster owner | NOT_FOUND | `PositionScheduleType::Shift` is only position attribute. |
| shift assignment | NOT_FOUND | Shift/Roster owner | NOT_FOUND | No assignment state found. |
| shift change | NOT_FOUND | Shift/Roster owner | NOT_FOUND | No change flow found. |
| shift swap | NOT_FOUND | Shift/Roster owner | NOT_FOUND | No swap flow found. |
| shift approval | NOT_FOUND | Roster owner + Access/Authority | NOT_FOUND | No approval flow found. |
| employee availability | Future ONB docs only | Availability/Scheduler owner | PARTIAL | No runtime storage found. |
| leave request | Adapter detects protected phrase only | Leave/HR/Scheduler owner | PARTIAL | No leave lifecycle found. |
| annual leave | Access journey docs future posture only | HR/Leave owner + Access consumes posture | PARTIAL | No runtime leave truth found. |
| sick leave | Access journey docs future posture only | HR/Leave owner + Access consumes posture | PARTIAL | No runtime sick leave truth found. |
| off-shift status | Access journey docs future posture only | Scheduler/Roster owner | PARTIAL | No runtime off-shift state found. |
| work pattern | PH1.POSITION schedule type only | Scheduler/Roster owner | PARTIAL | FullTime/PartTime/Contract/Shift enum is not a work pattern engine. |
| public holiday / regional calendar | Reminder journey future design only | Calendar/Holiday owner + PH1.E/Search when fresh data needed | NOT_FOUND | No public holiday table/logic found. |
| overtime | Position journey future design only | Payroll/Compensation/Timesheet owner consumes Scheduler facts | NOT_FOUND | No overtime calculation found. |
| breaks | NOT_FOUND | Scheduler/Attendance/Compliance owner | NOT_FOUND | No break state found. |
| clock-in / clock-out | Position journey future handoff only | Attendance/Timesheet owner | NOT_FOUND | No clock table/route found. |
| attendance | NOT_FOUND | Attendance owner | NOT_FOUND | Native availability strings are wake/profile availability, not attendance. |
| timesheet | NOT_FOUND | Timesheet owner | NOT_FOUND | No timesheet table/route found. |
| contractor hour logging | Position journey future handoff only | Contractor Time/Task/Scheduler + Finance/AP owner | NOT_FOUND | No contractor time runtime found. |
| contractor overrun tracking | Position journey future handoff only | Contractor Time/Finance/AP owner + PH1.REM reminders | NOT_FOUND | No overrun state found. |
| schedule reminders | PH1.REM reminder timing; calendar draft meeting reminder | PH1.REM owns timing; Scheduler owns source truth | PARTIAL | Shift/roster reminders not proven. |
| roster notifications | Future BCAST docs only | PH1.BCAST / PH1.DELIVERY | NOT_FOUND | No live roster notification path found. |
| payroll handoff | NOT_FOUND | Payroll/Timesheet owner | NOT_FOUND | No approved time facts or payroll integration found. |
| finance/budget handoff | NOT_FOUND | Finance/Budget owner | NOT_FOUND | No staffing cost/workload handoff found. |
| position handoff | `PositionScheduleType` and future docs | PH1.POSITION provides requirements; Scheduler owns allocation | PARTIAL | Schedule type only. |
| onboarding handoff | future ONB journey docs | PH1.ONB collects setup; Scheduler consumes after simulation | PARTIAL | No runtime Scheduler receiver found. |
| access off-work posture | future Access journey docs | Access consumes Scheduler/HR truth | PARTIAL | No runtime off-work signal found. |
| Desktop rendering | no workforce schedule UI found | Desktop render-only | NOT_FOUND | Native timer/wake scheduling is unrelated. |
| iPhone rendering | no workforce schedule UI found | iPhone render-only | NOT_FOUND | No roster UI found. |
| Adapter transport | calendar draft and protected-shadow heuristics | Adapter transport-only | PARTIAL | Adapter must not become scheduler authority. |
| audit/provenance | PH1.SCHED contract reason codes; WorkOrder ledger; reminder row for calendar draft | Scheduler/Roster audit owner + PH1.J | PARTIAL | Workforce schedule audit not found. |
| storage/migrations | WorkOrder/position/reminder storage only | Scheduler/Roster tables if implemented later | PARTIAL | No workforce scheduler tables found. |
| old compatibility paths | adapter roster/leave phrase heuristics; docs future references | canonical Scheduler/Roster future owner | PARTIAL | Active-caller checks needed before implementation. |

## 4. Current Scheduler / Roster / Workload Lifecycles

### A. Schedule Lifecycle

Repo truth supports an operational PH1.SCHED lifecycle:

- owner: `PH1.SCHED`.
- symbols/files: `SchedPolicyEvaluateRequest`, `SchedPolicyEvaluateOk`, `SchedDecisionComputeRequest`, `SchedDecisionComputeOk`, `SchedDecisionAction::{RetryAt, Fail, Wait}`, `Ph1SchedRuntime`, `Ph1SchedWiring`.
- inputs: `tenant_id`, `work_order_id`, `step_id`, `now_ns`, `step_started_at_ns`, `timeout_ms`, `max_retries`, `retry_backoff_ms`, `attempt_index`, optional `last_failure_reason_code`, retryable reason codes, `wait_is_pause_only`.
- outputs: deterministic retry eligibility, next attempt index, `RetryAt` with `next_due_at_ns`, `Fail`, or `Wait`.
- state changes: PH1.SCHED itself owns no table writes; WorkOrder orchestration reflects outcomes through append-only WorkOrder state outside the engine.
- audit evidence: reason codes and WorkOrder ledger fields exist for operational scheduling; PH1.SCHED docs require deterministic/auditable decisions.
- gaps: this is not workforce schedule lifecycle. Create/update/cancel human schedule is NOT_FOUND.

Repo truth supports a calendar-draft path:

- owner: PH1.N/PH1.X/SimulationExecutor/PH1.REM split.
- symbols/files: `IntentType::CreateCalendarEvent`, `normalize_calendar_event`, `confirm_text`, `CalendarDraftCreated`, `ReminderType::Meeting`, `CALENDAR_EVENT_CREATE`.
- inputs: natural-language calendar-event phrase with `when` and optional `person`; access instance with `CALENDAR_EVENT_CREATE`; active simulation registration.
- outputs: PH1.REM meeting reminder row and `CalendarDraftCreated { reminder_id }`.
- state changes: reminder row is created; no external calendar is written.
- audit evidence: reminder storage and access gate evidence exist; explicit app response says draft only.
- gaps: no external calendar connector, no attendee invite, no live calendar provider, no workforce schedule truth.

### B. Roster Lifecycle

Roster lifecycle is NOT_FOUND in runtime repo truth.

- create draft roster: NOT_FOUND.
- assign people: NOT_FOUND.
- publish roster: NOT_FOUND.
- update roster: NOT_FOUND.
- notify affected people: NOT_FOUND.
- handle shift changes/swaps: NOT_FOUND.
- audit: AUDIT_GAP.

Future docs mention roster group and scheduler/roster boundaries, but no canonical roster owner, records, transitions, or tests were found.

### C. Shift Lifecycle

Shift lifecycle is NOT_FOUND in runtime repo truth.

- create shift: NOT_FOUND.
- assign shift: NOT_FOUND.
- accept/acknowledge shift: NOT_FOUND.
- change shift: NOT_FOUND.
- swap shift: NOT_FOUND.
- complete shift: NOT_FOUND.
- audit: AUDIT_GAP.

`PositionScheduleType::Shift` only says that a position may be shift-based; it does not create shift truth.

### D. Availability / Leave Lifecycle

Availability and leave lifecycle is NOT_FOUND in runtime repo truth.

- submit availability: NOT_FOUND.
- request leave: NOT_FOUND.
- approve/deny leave: NOT_FOUND.
- mark sick/unavailable: NOT_FOUND.
- update access posture: PARTIAL future design only.
- notify supervisor: NOT_FOUND.
- audit: AUDIT_GAP.

Adapter protected-shadow heuristics detect phrases such as leave approval or roster, but detection is not leave execution or leave state.

### E. Attendance / Timesheet Lifecycle

Attendance and timesheet lifecycle is NOT_FOUND in runtime repo truth.

- clock in: NOT_FOUND.
- clock out: NOT_FOUND.
- break: NOT_FOUND.
- submit timesheet: NOT_FOUND.
- approve timesheet: NOT_FOUND.
- payroll handoff: NOT_FOUND.
- audit: AUDIT_GAP.

No attendance, clock, break, or timesheet table/route/test was found.

### F. Contractor Time Lifecycle

Contractor time lifecycle is NOT_FOUND in runtime repo truth.

- define estimated hours: NOT_FOUND.
- log time: NOT_FOUND.
- track progress: NOT_FOUND.
- detect overrun: NOT_FOUND.
- notify manager: NOT_FOUND.
- approve extra hours: NOT_FOUND.
- finance/AP handoff: NOT_FOUND.
- audit: AUDIT_GAP.

The Position Journey document describes future contractor scheduling and overrun handoffs, but current runtime proof was not found.

## 5. Data Model / Contracts / Packets

### Request Structs

| Struct / Equivalent | File / Path | Status | Notes |
| --- | --- | --- | --- |
| `SchedRequestEnvelope` | `crates/selene_kernel_contracts/src/ph1sched.rs` | FOUND | Bounded envelope for PH1.SCHED. |
| `SchedPolicyEvaluateRequest` | same | FOUND | Operational retry policy evaluation request. |
| `SchedDecisionComputeRequest` | same | FOUND | Operational next action compute request. |
| `SchedTurnInput` | `crates/selene_os/src/ph1sched.rs` | FOUND | OS wiring input for PH1.SCHED. |
| `WorkOrderLedgerEventInput` | `crates/selene_kernel_contracts/src/ph1work.rs` | EQUIVALENT_FOUND | Operational WorkOrder event input; not workforce workload. |
| `IntentType::CreateCalendarEvent` normalized draft | `crates/selene_engines/src/ph1n.rs` | EQUIVALENT_FOUND | Calendar-draft request equivalent. |
| workforce schedule create/update/cancel request | not found | NOT_FOUND | No canonical schedule packet found. |
| roster create/publish/update request | not found | NOT_FOUND | No canonical roster packet found. |
| shift assignment/change/swap request | not found | NOT_FOUND | No canonical shift packet found. |
| leave request | not found | NOT_FOUND | No leave packet found. |
| attendance/timesheet request | not found | NOT_FOUND | No attendance/timesheet packet found. |
| contractor time request | not found | NOT_FOUND | No contractor time packet found. |

### Response Structs

| Struct / Equivalent | File / Path | Status | Notes |
| --- | --- | --- | --- |
| `SchedPolicyEvaluateOk` | `crates/selene_kernel_contracts/src/ph1sched.rs` | FOUND | Retry eligibility and guard flags. |
| `SchedDecisionComputeOk` | same | FOUND | `RetryAt`, `Fail`, or `Wait` decision. |
| `SchedRefuse` | same | FOUND | Contract/budget/input refusal. |
| `Ph1SchedResponse` | same | FOUND | Union of scheduler outputs. |
| `SchedWiringOutcome` | `crates/selene_os/src/ph1sched.rs` | FOUND | `NotInvokedDisabled`, `Refused`, or `Forwarded`. |
| `SimulationDispatchOutcome::CalendarDraftCreated` | `crates/selene_os/src/simulation_executor.rs` | EQUIVALENT_FOUND | Calendar draft result with reminder id. |
| roster/shift/leave/timesheet response | not found | NOT_FOUND | No workforce response contracts found. |

### Records

| Record | File / Path | Status | Notes |
| --- | --- | --- | --- |
| `WorkOrderLedgerEvent` | `crates/selene_kernel_contracts/src/ph1work.rs` | FOUND | Operational work-order ledger row equivalent. |
| WorkOrder tables | `docs/04_KERNEL_CONTRACTS.md` | FOUND | `work_order_ledger`, `work_orders_current`, `work_order_step_attempts`, `work_order_leases`. |
| `PositionRecord.schedule_type` | `crates/selene_kernel_contracts/src/ph1position.rs` | PARTIAL | Job schedule type only. |
| PH1.REM reminder row | `crates/selene_storage/src/ph1f.rs` | PARTIAL | Used by calendar draft and reminders, not roster truth. |
| schedule ids | not found | NOT_FOUND | No workforce schedule id found. |
| roster ids | not found | NOT_FOUND | No roster id found. |
| shift ids | not found | NOT_FOUND | No shift id found. |
| leave ids | not found | NOT_FOUND | No leave id found. |
| timesheet ids | not found | NOT_FOUND | No timesheet id found. |
| attendance ids | not found | NOT_FOUND | No attendance id found. |
| contractor time ids | not found | NOT_FOUND | No contractor time id found. |

### Enums

| Enum | Values / Notes | Status | Notes |
| --- | --- | --- | --- |
| `SchedCapabilityId` | `SchedPolicyEvaluate`, `SchedDecisionCompute` | FOUND | PH1.SCHED capability enum. |
| `SchedDecisionAction` | `RetryAt`, `Fail`, `Wait` | FOUND | Operational scheduler actions. |
| `WorkOrderStatus` | `Draft`, `Clarify`, `Confirm`, `Executing`, `Done`, `Refused`, `Failed` | FOUND | WorkOrder status, not workforce workload. |
| `PositionScheduleType` | `FullTime`, `PartTime`, `Contract`, `Shift` | PARTIAL | Position schedule attribute. |
| schedule/roster/shift/leave/attendance state enums | not found | NOT_FOUND | Future owner required. |

### Error Types And Reason Codes

| Error / Reason | Evidence | Status | Notes |
| --- | --- | --- | --- |
| `PH1_SCHED_INPUT_SCHEMA_INVALID` | `crates/selene_engines/src/ph1sched.rs` | FOUND | Scheduler contract invalid. |
| `PH1_SCHED_UPSTREAM_INPUT_MISSING` | same | FOUND | Missing scheduler input. |
| `PH1_SCHED_BUDGET_EXCEEDED` | same | FOUND | Retryable reason/backoff bounds exceeded. |
| `PH1_SCHED_INTERNAL_PIPELINE_ERROR` | same | FOUND | Internal construction failure. |
| `SCHED_RETRY_SCHEDULED` | same | FOUND | RetryAt decision. |
| `SCHED_MAX_RETRIES_REACHED` | same | FOUND | Fail decision. |
| `SCHED_TIMEOUT` | same | FOUND | Fail decision. |
| `SCHED_NOT_RETRYABLE` | same | FOUND | Wait/fail posture. |
| workforce conflict/overtime/leave/payroll error codes | not found | NOT_FOUND | Future owner required. |

### Migration Tables

| Table / Migration | Evidence | Status | Notes |
| --- | --- | --- | --- |
| WorkOrder core tables | `0002_work_orders_core.sql`, `docs/04_KERNEL_CONTRACTS.md` | FOUND | Operational orchestration. |
| position requirements schema tables | `0014_position_requirements_schema_and_backfill_tables.sql` | PARTIAL | Position/onboarding, not roster. |
| access instance tables | `0009_access_instance_tables.sql`, `0015_access_master_schema_tables.sql`, `0016_access_ap_authoring_review_tables.sql` | PARTIAL | Access posture, not scheduler truth. |
| reminder storage | in-memory/store code and PH1.REM paths | PARTIAL | Reminder timing only. |
| workforce scheduler/roster/shift/leave/attendance/timesheet tables | not found | NOT_FOUND | SQL persistence gap. |

## 6. Product Function Types

| Product Function | Evidence Found | Current Behavior | Owner | Security / Compliance Implications | Missing Design |
| --- | --- | --- | --- | --- | --- |
| personal calendar event | `IntentType::CreateCalendarEvent`, `CalendarDraftCreated` | Creates PH1.REM meeting reminder; draft only | PH1.N/PH1.X/SimulationExecutor/PH1.REM | Requires `CALENDAR_EVENT_CREATE` access | External calendar provider, attendees, update/cancel. |
| meeting schedule | same | Draft-only meeting reminder | same | Access-gated | No live meeting/calendar owner. |
| work schedule | NOT_FOUND | none | Future Scheduler | Protected business action | Full lifecycle missing. |
| employee roster | future docs only | none | Future Roster | Protected staffing data | Full lifecycle missing. |
| roster group | ONB/Position journey docs | none | Future Roster | Access and privacy scoped | No canonical record. |
| shift assignment | NOT_FOUND | none | Future Roster/Shift | Protected staffing action | Missing. |
| shift swap | NOT_FOUND | none | Future Roster/Shift | Approval/compliance required | Missing. |
| shift publish | NOT_FOUND | none | Future Roster + BCAST/DELIVERY | Worker notification and audit | Missing. |
| staff availability | ONB future docs | none | Future Availability/Scheduler | Privacy-sensitive | Missing. |
| annual leave | Access journey future docs; adapter shadow detection | none | HR/Leave/Scheduler | Payroll/access posture | Missing. |
| sick leave | Access journey future docs | none | HR/Leave/Scheduler | Health/privacy-sensitive | Missing. |
| unavailable/off-shift | Access journey future docs | none | Scheduler/Roster | Access posture impact | Missing. |
| clock-in/clock-out | Position journey future docs | none | Attendance | Geolocation/worker privacy | Missing. |
| breaks | NOT_FOUND | none | Attendance/Compliance | Labor compliance | Missing. |
| overtime | Position journey future docs | none | Timesheet/Payroll/Compliance | Pay/compliance risk | Missing. |
| timesheet | NOT_FOUND | none | Timesheet | Payroll handoff risk | Missing. |
| contractor time tracking | Position journey future docs | none | Contractor Time/Task/Finance/AP | Contract/payment risk | Missing. |
| contractor overrun tracking | Position journey future docs | none | Contractor Time/Finance/AP + PH1.REM | Budget/payment risk | Missing. |
| schedule reminder | PH1.REM generic; calendar draft meeting reminder | reminder only | PH1.REM | Must not mutate schedule | Shift/roster source truth missing. |
| roster notification | BCAST docs future | none | PH1.BCAST/DELIVERY | Consent/channel/audit | Missing. |
| payroll timesheet handoff | NOT_FOUND | none | Payroll/Timesheet | Financial compliance | Missing. |
| finance workload/cost handoff | Position journey future docs | none | Finance/Budget | Budget truth | Missing. |
| position schedule requirement | `PositionScheduleType` | Position stores schedule type | PH1.POSITION | Must not grant schedule truth | Roster handoff missing. |
| onboarding schedule/roster group setup | ONB future docs | docs-only | PH1.ONB + Scheduler/Roster future | Must not mutate roster directly | Runtime receiver missing. |

## 7. Interaction With PH1.POSITION

PH1.POSITION currently stores schedule type only.

Evidence:

- `PositionScheduleType::{FullTime, PartTime, Contract, Shift}` in `crates/selene_kernel_contracts/src/ph1position.rs`.
- `PositionRecord.schedule_type` in `crates/selene_kernel_contracts/src/ph1position.rs`.
- `schedule_type` in `docs/04_KERNEL_CONTRACTS.md`.
- position storage/repo references in `crates/selene_storage/src/ph1f.rs` and `crates/selene_storage/src/repo.rs`.
- PH1.POSITION repo-truth extraction marks roster group, availability, schedule group, work-location registry, shift pattern, and roster truth as missing.

What is active:

- Position can carry a schedule type value.
- Position can participate in onboarding requirements and access/compensation references.

What is partial:

- Position can describe whether the position is full-time, part-time, contract, or shift-based.
- Position journey docs define Scheduler/Roster as the future owner for shifts, roster groups, workload, and leave/off-shift truth.

What is missing:

- actual shifts.
- roster group record.
- schedule group record.
- availability.
- shift pattern.
- work location registry.
- position-to-roster handoff runtime.
- position retirement impact on active schedules.

Critical rule:

PH1.POSITION may define expected schedule/roster requirements, but Scheduler/Roster must own actual work allocation.

## 8. Interaction With PH1.ONB

Current runtime evidence does not prove Scheduler/Roster mutation from PH1.ONB.

Evidence:

- ONB journey design lists future sender-known fields such as work location and roster/schedule group.
- ONB journey design says onboarding may pass approved start date, location, role, roster group, and availability to Scheduler/Roster only through its canonical owner and simulation.
- ONB journey design says no roster/schedule mutation from PH1.ONB alone.
- Runtime `crates/selene_os/src/ph1onb.rs` uses `PositionScheduleType` on onboarding-related position setup, but no canonical Scheduler/Roster receiver was found.

What is active:

- PH1.ONB can coordinate onboarding and consume position requirement schemas.
- PH1.ONB can carry position schedule type in related flows.

What is partial:

- future docs show the intended schedule/roster handoff boundary.

What is missing:

- onboarding availability collection in runtime.
- roster group runtime storage.
- schedule group runtime storage.
- first shift scheduling.
- workload update.
- Scheduler/Roster simulation and receiver.

Critical rule:

PH1.ONB collects/coordinated setup fields but must not mutate roster/schedule without Scheduler/Roster owner and simulation.

## 9. Interaction With Master Access

Repo evidence does not prove live Scheduler/Roster-to-Access work status integration.

Evidence:

- Master Access journey docs define leave/off-shift access posture as future architecture.
- Adapter protected-shadow heuristics treat roster/leave/payroll/salary/refund/access as protected-action shadows.
- `SimulationExecutor` calendar draft path requires `CALENDAR_EVENT_CREATE` access and owner/admin role posture.

What is active:

- Access gates calendar draft creation.
- Adapter can detect roster and leave approval as protected-risk language and fail closed in guarded contexts.

What is partial:

- Access future docs state Scheduler/Roster owns work availability, shifts, and roster truth; Access consumes that truth for posture.

What is missing:

- leave/off-shift truth provider.
- Scheduler/Roster notification to Access.
- Access policy consuming roster/shift status.
- high-risk action restriction based on off-shift/leave.

Critical rule:

Scheduler/Roster owns work availability truth. Access owns permission posture. Access may consume scheduler truth for leave/off-shift policy.

## 10. Interaction With PH1.REM

Repo evidence proves PH1.REM reminder timing, not schedule truth.

Evidence:

- `SimulationExecutor` maps `IntentType::CreateCalendarEvent` to `Ph1RemRequest::schedule_commit_v1` with `ReminderType::Meeting`.
- PH1.REM repo-truth extraction says task/scheduler/roster integration is not found.
- PH1.REM journey design states PH1.REM must not mutate tasks, schedules, rosters, or workloads.

What is active:

- PH1.REM can create reminders and meeting reminders.
- calendar draft creates a meeting reminder.

What is partial:

- schedule reminder integration exists only as generic reminder timing and calendar draft.

What is missing:

- shift reminders from source schedule truth.
- roster reminders.
- missed clock-in/out follow-up.
- contract expiry or shift change reminders from Scheduler/Roster.

Critical rule:

PH1.REM owns reminder timing. Scheduler/Roster owns schedule truth.

## 11. Interaction With PH1.BCAST / PH1.DELIVERY

Repo evidence does not prove workforce roster notifications.

Evidence:

- BCAST/DELIVERY journey docs define future task/scheduler/roster handoff boundaries.
- Advanced delivery docs warn not to silently convert message/read receipt into task, roster, or schedule completion.
- No roster-publish delivery route or shift assignment delivery route was found.

What is active:

- PH1.BCAST/DELIVERY exists for broadcast/delivery contexts, separate from scheduling.

What is partial:

- future docs identify notification boundaries.

What is missing:

- roster change broadcast.
- shift assignment delivery.
- leave approval/denial notification.
- schedule reminder external delivery from Scheduler/Roster.
- delivery proof for roster notifications.

Critical rule:

Scheduler/Roster must not become delivery provider.

## 12. Interaction With Payroll / HR / Compensation / Finance

Runtime evidence does not prove payroll, HR, compensation, or finance handoff from schedule/roster facts.

Evidence:

- Position journey design says Scheduler/Roster owns time while Payroll/HR/Compensation/Finance own pay and money truth.
- Position repo-truth extraction states Position does not own payroll, salary, schedule, roster, workload, leave, termination, or resignation truth.
- No timesheet, attendance, overtime, leave, or contractor-hour table was found.

What is active:

- Position can reference compensation band and schedule type.

What is partial:

- future docs define payroll/HR/scheduler boundaries.

What is missing:

- approved time facts.
- payroll schedule handoff.
- timesheet handoff.
- overtime feed.
- leave feed.
- contractor hours feed to Finance/AP.
- roster staffing cost forecast.
- shift premiums/allowances.

Critical rule:

Scheduler/Roster may produce approved time/work facts. Payroll/Compensation/Finance own pay, calculation, disbursement, budget, and financial truth.

## 13. Time, Timezone, Public Holiday, And Labor Compliance

Repo evidence for workforce time compliance is missing.

Found:

- PH1.SCHED uses `MonotonicTimeNs` for deterministic retry/wait/fail decisions.
- PH1.REM uses reminder timing and timezone fields for reminders.
- calendar draft path hardcodes timezone `"UTC"` when creating a meeting reminder.
- PH1.POSITION has `jurisdiction` and `PositionScheduleType`.

Not found:

- workforce timezone policy.
- roster timezone.
- daylight saving handling for shifts.
- regional public holiday tables.
- labor law compliance.
- break rules.
- overtime rules.
- maximum hours.
- minimum rest.
- weekend rules.
- industry awards/agreements.
- country/state/province rule owner.

Correct future rule:

Scheduler/Roster may use calendar/compliance owners. PH1.E/Search or approved providers may verify current public holiday/labor data. Scheduler/Roster must not invent legal/payroll rules.

## 14. Contractor Scheduling And Workload

Current contractor scheduling/hour support is NOT_FOUND.

Evidence:

- `PositionScheduleType::Contract` exists.
- Position Journey design describes contractor billing mode, time/progress/overrun handoffs, geolocation consent, and Finance/AP boundaries.
- No runtime contractor-hour, estimated-hour, overrun, manager-approval, or Finance/AP handoff table/route was found.

Current answers:

- Can contractors have schedules? UNKNOWN / NOT_FOUND.
- Can contractors log hours? NOT_FOUND.
- Can contractor estimated hours be tracked? NOT_FOUND.
- Can overrun alerts be generated? NOT_FOUND.
- Can manager approve extra hours? NOT_FOUND.
- Can contractor time feed Finance/AP? NOT_FOUND.
- Does repo support geolocation/site validation? NOT_FOUND for contractor scheduling; device/wake/local availability surfaces are unrelated.

## 15. PH1.D / GPT-5.5 / PH1.N / PH1.X Interaction

Current repo has limited probabilistic-adjacent schedule understanding only for calendar draft, not workforce scheduler/roster.

Active evidence:

- PH1.N normalizes `IntentType::CreateCalendarEvent`.
- PH1.X confirms `CreateCalendarEvent` and dispatches a simulation candidate.
- SimulationExecutor converts confirmed calendar event into a PH1.REM meeting reminder.

Partial / missing:

- PH1.N does not prove roster/shift/leave/attendance/timesheet extraction.
- PH1.D/GPT-5.5 proposal path for schedule/roster is not found.
- PH1.X protected-shadow logic exists via adapter and general request lattice docs, but no canonical Scheduler/Roster route exists.

Correct future rule:

Users may say:

- "Roster Tom every Friday morning."
- "Give Sarah next week off."
- "Move the warehouse team to 7 AM shifts."
- "Find someone to cover Tim tomorrow."
- "Remind the contractor to log hours."
- "Make sure nobody exceeds 38 hours this week."

OpenAI/GPT-5.5 may help understand, propose candidates, and ask clarification. PH1.N extracts time, people, shift, location, role, constraints, recurrence, and conflict. PH1.X validates route, risk, owner, protected effects, and confirmation. Scheduler/Roster executes only deterministic validated schedule actions.

Future proposal path needed:

1. PH1.D/GPT-5.5 schedule/roster proposal shell.
2. PH1.N schedule slot extraction.
3. PH1.X Scheduler/Roster route/risk validation.
4. deterministic Scheduler/Roster simulation discovery.
5. PH1.WRITE clarification and confirmation wording.
6. Scheduler/Roster commit only after access, authority, simulation, and audit gates.

## 16. PH1.WRITE Interaction

Current scheduler/roster guidance wording is not PH1.WRITE-owned in runtime evidence.

Found:

- PH1.X hardcodes calendar confirmation text for `CreateCalendarEvent`.
- App ingress hardcodes calendar draft response text.
- Adapter protected-shadow heuristics use hardcoded protected phrases for roster/leave risk.

Not found:

- PH1.WRITE-owned roster change wording.
- PH1.WRITE-owned scheduling conflict explanation.
- PH1.WRITE-owned leave response.
- PH1.WRITE-owned clock-in/out messages.
- PH1.WRITE-owned overtime warning.
- PH1.WRITE-owned roster notification text.

Risk markers:

- SCHEDULER_WRITING_OWNER_RISK: PARTIAL. Calendar draft text is hardcoded outside PH1.WRITE.
- ROSTER_WRITING_OWNER_RISK: NOT_FOUND / DESIGN_GAP.
- HARDCODED_SCHEDULE_GUIDANCE_RISK: FOUND for calendar draft and confirmation.
- CLIENT_SCHEDULE_TEXT_RISK: NOT_FOUND for workforce scheduler; native strings are not workforce schedule decisions.
- ADAPTER_SCHEDULE_TEXT_RISK: PARTIAL. Adapter uses hardcoded protected-shadow roster/leave labels.

Correct future rule:

PH1.WRITE owns final wording for roster changes, scheduling questions, conflict explanations, leave responses, clock-in/out messages, overtime warnings, and notifications.

## 17. Desktop / iPhone / Adapter Boundaries

### Desktop

Found:

- Desktop has native scheduling strings for timers, idle close, wake profile availability refresh, and local session/wake behavior.
- No workforce schedule/roster UI, shift action surface, attendance surface, timesheet surface, leave surface, or roster publish UI was found.

Status:

- DESKTOP_SCHEDULER_AUTHORITY_RISK: NOT_FOUND for workforce scheduler; unrelated native timer surfaces must remain non-authoritative.
- DESKTOP_ROSTER_AUTHORITY_RISK: NOT_FOUND.

### iPhone

Found:

- No iPhone workforce schedule/roster UI was found in searched surfaces.

Status:

- IPHONE_SCHEDULER_AUTHORITY_RISK: NOT_FOUND.
- IPHONE_ROSTER_AUTHORITY_RISK: NOT_FOUND.

### Adapter

Found:

- Adapter calendar draft test drives confirmation and persists a meeting reminder.
- Adapter seed helper grants `CALENDAR_EVENT_CREATE`.
- Adapter protected-shadow heuristics mark roster/leave/payroll/salary/refund/access language as protected or simulation-candidate.

Status:

- ADAPTER_SCHEDULER_AUTHORITY_RISK: PARTIAL. Calendar draft transport exists, and roster/leave classification strings exist; no actual scheduler execution was found.
- ADAPTER_ROSTER_AUTHORITY_RISK: PARTIAL. Roster detection must remain risk classification only, not authority.

Boundary rule:

Desktop and iPhone must render/submits bounded inputs only. Adapter must transport and classify bounded inputs only. Neither client nor Adapter may decide schedule/roster truth.

## 18. Security / Privacy / Governance Model

Repo evidence for workforce scheduler security/governance is mostly missing.

| Governance Topic | Repo Evidence | Status | Notes |
| --- | --- | --- | --- |
| who can create schedules | calendar draft requires `CALENDAR_EVENT_CREATE`; workforce schedule missing | PARTIAL | Calendar only. |
| who can update schedules | not found | NOT_FOUND | SECURITY_GAP. |
| who can publish rosters | not found | NOT_FOUND | SECURITY_GAP. |
| who can assign shifts | not found | NOT_FOUND | SECURITY_GAP. |
| who can approve swaps | not found | NOT_FOUND | SECURITY_GAP. |
| who can approve leave | adapter detects leave approval phrase only | PARTIAL | No leave authority. |
| who can edit timesheets | not found | NOT_FOUND | SECURITY_GAP. |
| who can approve timesheets | not found | NOT_FOUND | SECURITY_GAP. |
| who can see schedules | not found | NOT_FOUND | SECURITY_GAP. |
| who can see leave/sick status | not found | NOT_FOUND | PRIVACY_GAP. |
| who can see contractor hours | not found | NOT_FOUND | SECURITY_GAP. |
| who can override work-hour limits | not found | NOT_FOUND | COMPLIANCE_GAP. |
| access gate | calendar draft access gate exists | PARTIAL | `CALENDAR_EVENT_CREATE` only. |
| authority gate | calendar event owner/admin check exists | PARTIAL | No roster authority. |
| simulation gate | calendar draft requires simulation; PH1.SCHED simulation catalog exists | PARTIAL | Workforce simulation missing. |
| confirmation requirements | PH1.X calendar confirmation exists | PARTIAL | No roster/leave confirmation policy. |
| audit | WorkOrder/reason codes/reminder row exist | PARTIAL | Workforce audit missing. |
| idempotency | PH1.SCHED and calendar draft idempotency evidence | PARTIAL | Workforce idempotency missing. |
| field sensitivity | Access/Position docs future | PARTIAL | No schedule field classification. |
| worker privacy | not found | NOT_FOUND | PRIVACY_GAP. |
| geolocation consent | Position journey future docs only | PARTIAL | No runtime geolocation scheduling. |
| labor compliance | not found | NOT_FOUND | COMPLIANCE_GAP. |

## 19. State Machines

RECONSTRUCTED_FROM_REPO_EVIDENCE.

### Operational PH1.SCHED State/Action Machine

Actual current actions:

- `RetryAt`: retry allowed, emits `next_due_at_ns`.
- `Fail`: timeout or max retries reached.
- `Wait`: not retryable or pause-only, must not advance attempt index.
- `Refuse`: validation, budget, or pipeline refusal.
- `NotInvokedDisabled`: OS wiring disabled.

This is implemented for WorkOrder retry/wait/fail posture.

### Schedule States

Workforce schedule states are NOT_FOUND.

Possible future states:

- Draft: NOT_FOUND.
- Active: NOT_FOUND.
- Published: NOT_FOUND.
- Updated: NOT_FOUND.
- Cancelled: NOT_FOUND.
- Completed: NOT_FOUND.
- Failed: NOT_FOUND.

Calendar draft equivalent:

- CalendarDraftCreated: PARTIAL, implemented as reminder-backed draft only.

### Roster States

- Draft: NOT_FOUND.
- PendingReview: NOT_FOUND.
- Published: NOT_FOUND.
- Changed: NOT_FOUND.
- Locked: NOT_FOUND.
- Cancelled: NOT_FOUND.
- Archived: NOT_FOUND.

### Shift States

- Open: NOT_FOUND.
- Assigned: NOT_FOUND.
- Accepted: NOT_FOUND.
- Declined: NOT_FOUND.
- Swapped: NOT_FOUND.
- Completed: NOT_FOUND.
- Missed: NOT_FOUND.
- Cancelled: NOT_FOUND.

### Leave States

- Requested: NOT_FOUND.
- Approved: NOT_FOUND.
- Denied: NOT_FOUND.
- Cancelled: NOT_FOUND.
- Active: NOT_FOUND.
- Completed: NOT_FOUND.

### Attendance States

- ClockedIn: NOT_FOUND.
- OnBreak: NOT_FOUND.
- ClockedOut: NOT_FOUND.
- Submitted: NOT_FOUND.
- Approved: NOT_FOUND.
- Rejected: NOT_FOUND.

### Contractor Time States

- Estimated: NOT_FOUND.
- Logged: NOT_FOUND.
- Submitted: NOT_FOUND.
- Approved: NOT_FOUND.
- OverrunWarning: NOT_FOUND.
- OverrunApproved: NOT_FOUND.
- Rejected: NOT_FOUND.

Do not claim runtime implements these workforce states without future repo proof.

## 20. Error Handling And Reason Codes

| Error / Reason Code | Evidence | Status | Notes |
| --- | --- | --- | --- |
| schedule not found | not found | NOT_FOUND | Workforce schedule missing. |
| roster not found | not found | NOT_FOUND | Roster missing. |
| shift not found | not found | NOT_FOUND | Shift missing. |
| worker unavailable | not found | NOT_FOUND | Availability missing. |
| leave conflict | not found | NOT_FOUND | Leave engine missing. |
| overtime violation | not found | NOT_FOUND | Compliance missing. |
| public holiday conflict | not found | NOT_FOUND | Holiday owner missing. |
| timezone invalid | PH1.REM/calendar draft uses timezone string but no workforce policy | PARTIAL | Workforce timezone missing. |
| invalid recurrence | PH1.REM has reminder recurrence checks | PARTIAL | No work-pattern recurrence. |
| access denied | calendar draft access gate | PARTIAL | `CALENDAR_EVENT_CREATE` only. |
| authority missing | calendar event non-owner role escalates | PARTIAL | No roster authority. |
| simulation missing | calendar event missing simulation fails closed | PARTIAL | No Scheduler/Roster simulation. |
| approval required | calendar event non-owner requires AP equivalent | PARTIAL | No shift/leave approval. |
| conflict detected | not found | NOT_FOUND | Roster conflict engine missing. |
| coverage insufficient | not found | NOT_FOUND | Roster coverage engine missing. |
| duplicate shift | not found | NOT_FOUND | Shift engine missing. |
| timesheet missing | not found | NOT_FOUND | Timesheet engine missing. |
| clock-in outside location | not found | NOT_FOUND | Geolocation/attendance missing. |
| geolocation denied | not found | NOT_FOUND | Consent policy missing. |
| contractor hours overrun | not found | NOT_FOUND | Contractor time missing. |
| payroll handoff blocked | not found | NOT_FOUND | Payroll handoff missing. |
| client route mismatch | adapter generic runtime can fail closed; no scheduler-specific route | PARTIAL | Future Adapter route proof needed. |

PH1.SCHED actual reason codes:

- `PH1_SCHED_OK_POLICY_EVALUATE`
- `PH1_SCHED_OK_DECISION_COMPUTE`
- `SCHED_RETRY_SCHEDULED`
- `SCHED_MAX_RETRIES_REACHED`
- `SCHED_TIMEOUT`
- `SCHED_NOT_RETRYABLE`
- `PH1_SCHED_INPUT_SCHEMA_INVALID`
- `PH1_SCHED_UPSTREAM_INPUT_MISSING`
- `PH1_SCHED_BUDGET_EXCEEDED`
- `PH1_SCHED_INTERNAL_PIPELINE_ERROR`
- `PH1_SCHED_VALIDATION_FAILED`

These reason codes are operational scheduler codes, not workforce scheduler/roster reason codes.

## 21. Audit / Provenance / Evidence

Audit evidence exists for operational orchestration and calendar draft, not for workforce Scheduler/Roster.

| Audit Question | Repo Evidence | Status | Notes |
| --- | --- | --- | --- |
| Is schedule creation audited? | calendar draft creates reminder row; WorkOrder retry scheduler is reason-coded | PARTIAL | Workforce schedule creation audit NOT_FOUND. |
| Is roster publish audited? | not found | AUDIT_GAP | No roster publish. |
| Is shift assignment audited? | not found | AUDIT_GAP | No shift assignment. |
| Is shift swap audited? | not found | AUDIT_GAP | No shift swap. |
| Is leave request/approval audited? | not found | AUDIT_GAP | No leave state. |
| Is clock-in/out audited? | not found | AUDIT_GAP | No attendance state. |
| Is timesheet approval audited? | not found | AUDIT_GAP | No timesheet. |
| Is overtime calculation audited? | not found | AUDIT_GAP / COMPLIANCE_GAP | No overtime. |
| Is payroll handoff audited? | not found | AUDIT_GAP | No handoff. |
| Is contractor hour logging audited? | not found | AUDIT_GAP | No contractor time. |
| Are tenant/workspace/company/position/user refs recorded? | PH1.SCHED has tenant/work_order; PH1.POSITION has tenant/company/position; calendar reminder has user/tenant | PARTIAL | Workforce refs missing; workspace missing. |
| Are location/geolocation refs recorded? | not found for scheduler | AUDIT_GAP / PRIVACY_GAP | No scheduling geolocation. |
| Are client/adapter schedule events audited? | adapter test proves calendar draft path; no workforce schedule event audit | PARTIAL | Adapter roster/leave execution missing. |

## 22. Current Tests / Smokes / Acceptance

| Test / Smoke | Path | What It Proves | What It Does Not Prove | Status |
| --- | --- | --- | --- | --- |
| `at_sched_01_retry_action_requires_next_due` | `crates/selene_kernel_contracts/src/ph1sched.rs` | `RetryAt` requires next due time. | Workforce scheduling. | FOUND |
| `at_sched_02_wait_action_must_not_advance_attempt` | same | WAIT cannot advance attempt index. | Workforce scheduling. | FOUND |
| `at_sched_03_retryable_reason_codes_must_be_unique` | same | Retryable reason codes unique. | Workforce scheduling. | FOUND |
| `at_sched_04_wait_pause_rule_is_mandatory` | same | Wait pause-only invariant. | Workforce scheduling. | FOUND |
| `at_sched_01_retry_schedule_is_deterministic` | `crates/selene_engines/src/ph1sched.rs` | Deterministic retry schedule. | Human work schedule. | FOUND |
| `at_sched_02_max_retries_enforced` | same | Max retries enforced. | Human work schedule. | FOUND |
| `at_sched_03_wait_does_not_advance_plan` | same | Wait does not advance plan. | Human work schedule. | FOUND |
| `at_sched_04_timeout_enforced` | same | Timeout produces fail posture. | Human work schedule. | FOUND |
| `at_sched_01_os_invokes_and_returns_retry_or_fail_or_wait` | `crates/selene_os/src/ph1sched.rs` | OS PH1.SCHED invocation. | Workforce scheduler. | FOUND |
| `at_sched_02_disabled_returns_not_invoked` | same | Disabled path. | Workforce scheduler. | FOUND |
| `at_sched_03_wait_does_not_advance_and_is_forwarded` | same | OS forwards valid WAIT. | Workforce scheduler. | FOUND |
| `at_sched_04_wait_advance_is_refused` | same | OS refuses invalid WAIT advance. | Workforce scheduler. | FOUND |
| `at_n_28_calendar_event_normalizes_from_create_calendar_event_phrase` | `crates/selene_engines/src/ph1n.rs` | Calendar-event NLP normalization. | External calendar or roster. | PARTIAL |
| `at_n_29_calendar_event_normalizes_from_schedule_meeting_phrase` | same | Meeting phrase maps to calendar event. | External calendar or roster. | PARTIAL |
| `at_x_confirm_yes_dispatches_simulation_candidate_for_calendar_event` | `crates/selene_os/src/ph1x.rs` | Calendar confirm dispatches simulation candidate. | Scheduler/Roster. | PARTIAL |
| `at_sim_exec_calendar_event_schedules_meeting_reminder_and_is_idempotent` | `crates/selene_os/src/simulation_executor.rs` | Calendar draft creates idempotent meeting reminder. | External calendar or roster. | PARTIAL |
| `at_sim_exec_calendar_event_non_owner_role_requires_ap` | same | Calendar draft access/role escalation. | Roster permissions. | PARTIAL |
| `at_sim_exec_calendar_event_missing_sim_registration_fails_closed` | same | Missing simulation fails closed. | Roster simulation. | PARTIAL |
| `run_a_response_text_for_calendar_draft_is_explicit_draft_only` | `crates/selene_os/src/app_ingress.rs` | User-facing calendar draft text says draft only. | External calendar. | PARTIAL |
| `at_adapter_03ba_calendar_event_confirm_yes_dispatches_sim_and_persists_meeting_reminder` | `crates/selene_adapter/src/lib.rs` | Adapter path persists meeting reminder after confirm. | Workforce scheduling. | PARTIAL |
| workforce roster tests | not found | None. | Roster lifecycle. | TEST_GAP |
| shift tests | not found | None. | Shift lifecycle. | TEST_GAP |
| leave/availability tests | not found | None. | Leave/availability lifecycle. | TEST_GAP |
| attendance/timesheet tests | not found | None. | Timekeeping. | TEST_GAP |
| contractor hours tests | not found | None. | Contractor workload. | TEST_GAP |

## 23. Old Paths / Compatibility / Wrong-Owner Risks

| Path / Symbol | Current Status | Correct Canonical Owner | Retirement Condition | Active-Caller Check Needed |
| --- | --- | --- | --- | --- |
| `PH1.SCHED` name | Active operational scheduler | Keep for WorkOrder retry; future workforce Scheduler/Roster needs explicit name/owner reconciliation | Grand Architecture Reconciliation locks naming | yes |
| `docs/08_SIMULATION_CATALOG.md` `SCHED_NEXT_ACTION_DRAFT` domain `Scheduler` | Active draft simulation catalog item | Operational Scheduler / WorkOrder orchestration | Clarify not workforce scheduling | yes |
| PH1.POSITION `PositionScheduleType` | Active position attribute | PH1.POSITION only | Scheduler/Roster owner receives separate handoff | yes |
| ONB future roster/schedule group docs | Design-only | PH1.ONB collects; Scheduler/Roster owns truth | Runtime handoff proof | yes |
| PH1.REM calendar draft meeting reminder | Active partial | PH1.REM timing; Calendar/Scheduler owns source truth | Calendar owner proof exists | yes |
| PH1.BCAST task/scheduler/roster docs | Design-only | BCAST/DELIVERY delivery only | Scheduler/Roster source event proof | yes |
| Access leave/off-shift posture docs | Design-only | Access consumes Scheduler/HR truth | Work-status truth owner proof | yes |
| adapter `h380_simulation_candidate` roster label | Active heuristic | PH1.X/Scheduler route validation | canonical route replaces heuristic | yes |
| adapter protected phrase `approve leave` | Active heuristic | Leave/HR/Scheduler + PH1.X | leave route proof | yes |
| client-side timer/wake/session scheduling strings | Active native behavior | Native shell only; not workforce schedule | no workforce authority leakage | yes |
| calendar draft response text in app ingress | Active hardcoded wording | PH1.WRITE future wording boundary | PH1.WRITE calendar/scheduler wording proof | yes |
| web search parallel scheduler | Active helper | Web search plan internal | never treated as Scheduler/Roster | yes |
| PH1.WORK "workload" confusion | Active WorkOrder ledger | PH1.WORK operational work orders | document naming boundary | yes |
| stale docs mentioning roster/leave as examples | Mixed | Future Scheduler/Roster/Leave owner | activation pack reconciles | yes |

Wrong-owner risks:

- Position storing schedule truth: PARTIAL risk via `PositionScheduleType`; must remain metadata only.
- ONB mutating roster/schedule: design explicitly forbids; runtime owner still missing.
- Reminder owning schedule truth: PH1.REM calendar draft must not become calendar/roster truth.
- Broadcast owning roster status: docs warn against this; runtime proof missing.
- Access deciding work availability: future Access must consume Scheduler/HR truth, not invent it.
- Payroll owning raw schedule state: no evidence, but future boundary must prevent it.
- client-side shift decisions: no current proof, but future clients must be render-only.
- adapter schedule shortcuts: roster/leave heuristics must not become authority.
- calendar draft pretending to be schedule truth: current text is good because it says draft-only; preserve this until real calendar owner exists.

## 24. Full Functionality Master List

| Functionality | Description | Evidence | Owner | Status | Future Action |
| --- | --- | --- | --- | --- | --- |
| PH1.SCHED policy evaluate | Evaluate timeout/retry eligibility for WorkOrder step | `SchedPolicyEvaluateRequest/Ok` | PH1.SCHED | FOUND | Keep as operational scheduler. |
| PH1.SCHED decision compute | Compute `RETRY_AT`, `FAIL`, or `WAIT` | `SchedDecisionComputeRequest/Ok` | PH1.SCHED | FOUND | Keep separate from workforce scheduling. |
| OS scheduler wiring | Invoke PH1.SCHED and forward/refuse output | `Ph1SchedWiring` | Selene OS | FOUND | Preserve fail-closed invariants. |
| WorkOrder retry persistence | WorkOrder ledger/current/attempt/lease fields | `docs/04_KERNEL_CONTRACTS.md` | PH1.WORK / OS core tables | FOUND | Do not confuse with workload engine. |
| create calendar draft | Confirm calendar intent and schedule meeting reminder | `CalendarDraftCreated`, `ReminderType::Meeting` | SimulationExecutor + PH1.REM | PARTIAL | Build real Calendar owner before live calendar claims. |
| calendar access gate | Require `CALENDAR_EVENT_CREATE` and owner/admin role | `enforce_calendar_event_create_gate` | Access + SimulationExecutor | PARTIAL | Future calendar/schedule permissions must be explicit. |
| calendar NLP normalization | Recognize create calendar event / schedule meeting | `normalize_calendar_event` | PH1.N | PARTIAL | Extend only through PH1.D/PH1.N route if approved. |
| calendar confirmation | Ask deterministic confirmation | `confirm_text` | PH1.X today; PH1.WRITE future | PARTIAL | Move user wording to PH1.WRITE boundary later. |
| position schedule type | Store FullTime/PartTime/Contract/Shift attribute | `PositionScheduleType` | PH1.POSITION | PARTIAL | Handoff to Scheduler/Roster future. |
| roster group | Group workers for roster | docs future only | Future Roster | NOT_FOUND | Define canonical model. |
| schedule group | Schedule grouping | docs future only | Future Scheduler | NOT_FOUND | Define canonical model. |
| create schedule | Create work schedule | not found | Future Scheduler | NOT_FOUND | Implement only after reconciliation. |
| update schedule | Change work schedule | not found | Future Scheduler | NOT_FOUND | Missing. |
| publish roster | Publish roster to workers | not found | Future Roster + BCAST | NOT_FOUND | Missing. |
| assign shift | Assign worker to shift | not found | Future Roster/Shift | NOT_FOUND | Missing. |
| swap shift | Worker/manager shift swap | not found | Future Roster/Shift | NOT_FOUND | Missing. |
| request leave | Request leave | adapter phrase only | Future Leave/HR/Scheduler | PARTIAL | Canonical leave lifecycle needed. |
| approve leave | Approve/deny leave | adapter phrase only | Future Leave/HR/Scheduler + Access | PARTIAL | Missing protected execution owner. |
| mark sick | Mark sick/unavailable | future docs only | Future Leave/HR/Scheduler | NOT_FOUND | Missing privacy model. |
| clock in | Start attendance | not found | Future Attendance | NOT_FOUND | Missing. |
| clock out | End attendance | not found | Future Attendance | NOT_FOUND | Missing. |
| submit timesheet | Submit time facts | not found | Future Timesheet | NOT_FOUND | Missing. |
| approve timesheet | Approve time facts | not found | Future Timesheet + Access/Authority | NOT_FOUND | Missing. |
| calculate overtime fact | Determine overtime facts | not found | Future Timesheet/Compliance | NOT_FOUND | Missing. |
| send roster notification | Notify affected workers | future BCAST docs only | BCAST/DELIVERY | NOT_FOUND | Needs source event from Roster. |
| schedule shift reminder | Reminder before shift | future PH1.REM docs only | PH1.REM timing + Scheduler source truth | NOT_FOUND | Missing source truth. |
| feed payroll time facts | Handoff approved time | not found | Timesheet -> Payroll | NOT_FOUND | Missing. |
| track contractor hours | Log contractor hours | future Position docs only | Contractor Time/Task/Scheduler | NOT_FOUND | Missing. |
| contractor overrun alert | Alert on overrun | future Position docs only | Contractor Time + PH1.REM/BCAST | NOT_FOUND | Missing. |

## 25. Comparison To Master Architecture

### PH1.POSITION Position Journey

Current PH1.POSITION stores schedule type and future docs describe Scheduler/Roster handoff. Repo truth does not support PH1.POSITION owning actual schedules, rosters, shifts, availability, leave, attendance, timesheets, or contractor time.

### PH1.ONB Onboarding Journey

ONB future docs allow collection of start date, location, roster group, and availability, but require handoff to Scheduler/Roster through canonical owner and simulation. Runtime proof for that owner is missing.

### Master Access Governance + Per-User Access Journey

Access future design depends on leave/off-shift truth for high-risk access posture. Runtime Scheduler/Roster work-status truth is missing, so Access integration remains design-only except calendar-draft access gating.

### PH1.REM Reminder Journey

PH1.REM can schedule reminders and meeting reminders. It must not own schedule/roster truth. Scheduler/Roster future owner must provide source facts before REM can remind about shifts or missed clock-ins.

### PH1.BCAST / PH1.DELIVERY

BCAST/DELIVERY should own outbound roster or shift notifications where needed. Current repo does not prove roster notification generation or delivery proof.

### Payroll/HR Future Owner

Payroll/HR must own employment, pay, leave policy, and payroll truth. Scheduler/Roster may later produce approved time/work facts, but current repo has no timesheet or payroll handoff.

### Compensation Future Owner

Compensation may use schedule, overtime, and contractor-hour facts later. Current repo has no compensation integration with Scheduler/Roster.

### Finance/Budget Future Owner

Finance/Budget may consume staffing cost and contractor overrun facts later. Current repo has no staffing cost or budget workload handoff.

### PH1.D Proposal Gateway

No PH1.D/GPT-5.5 schedule proposal path was found. Future implementation must keep model proposals non-authoritative.

### PH1.N Meaning Unravelling

PH1.N has calendar event normalization. Workforce schedule, roster, leave, shift, attendance, timesheet, and contractor-time extraction are missing.

### PH1.X Request Decision Lattice

PH1.X/adapter surfaces can classify protected roster-like language, and PH1.X handles calendar confirmation. A canonical Scheduler/Roster route and risk matrix is missing.

### PH1.WRITE Human Presentation

Calendar draft/confirmation wording is currently hardcoded in PH1.X/app ingress. Future scheduler/roster wording should be PH1.WRITE-owned.

### Identity + Access + Authority Spine

Calendar draft has access gating. Workforce scheduler/roster access, authority, approvals, simulations, and audit are missing.

### Tenant / Workspace Governance

PH1.SCHED and Position carry tenant/company context. Workforce workspace/department/location/shift scope is missing.

### Desktop/iPhone Render-Only Boundary

No workforce UI found. Future Desktop/iPhone surfaces must be render-only and submit bounded inputs to canonical runtime owners.

### Adapter Transport-Only Boundary

Adapter has calendar draft path and roster/leave heuristic language. Future Adapter must not decide roster/schedule truth.

### Old Compatibility Path Retirement

Active-caller checks are needed before changing adapter protected-shadow heuristics, calendar draft semantics, or any docs that use Scheduler as a broad domain label.

## 26. Gaps / Missing Pieces

| Gap | Evidence | Risk | Recommended Future Action | Priority |
| --- | --- | --- | --- | --- |
| PH1.SCHED name collision | PH1.SCHED exists as WorkOrder retry scheduler | future workforce scheduler may collide with existing engine | Grand Architecture Reconciliation naming/owner decision | High |
| missing standalone Scheduler engine | no `scheduler`/`ph1scheduler` runtime files found | schedule requests may be misrouted to PH1.SCHED/REM | create canonical Scheduler owner after activation | High |
| missing standalone Roster engine | no roster runtime/storage/tests | roster mutations could drift into Position/Adapter | define Roster contract/state machine | High |
| missing Workload engine | PH1.WORK is WorkOrder ledger only | workload/capacity confused with orchestration work | define workforce Workload owner separately | Medium |
| missing shift lifecycle | no shift records/states | cannot assign or swap shifts safely | build shift lifecycle slice | High |
| missing roster publish flow | no roster publish path | workers cannot receive authoritative roster changes | build roster publish/change/notification handoff | High |
| missing availability | future docs only | scheduling conflicts and privacy gaps | add availability model and access controls | High |
| missing leave/sick flow | adapter phrase only | protected HR actions may be mishandled | add Leave/HR/Scheduler owner boundary | High |
| missing clock-in/out | no attendance runtime | payroll/time facts absent | add attendance proof | High |
| missing timesheet | no timesheet runtime | payroll handoff impossible | add timesheet proof | High |
| missing contractor hours | future docs only | contractor payments/overruns unmanaged | add contractor time flow | Medium |
| missing overtime/breaks/public holiday logic | no compliance tables | labor/payroll compliance risk | add compliance owner handoff | High |
| missing timezone/DST policy | calendar draft uses UTC reminder | wrong shift/reminder times | add timezone/locality policy | High |
| missing position-to-roster handoff | Position schedule type only | Position may be misused as roster truth | build handoff map | Medium |
| missing onboarding schedule handoff | ONB docs only | onboarding may duplicate scheduler logic | build ONB-to-Scheduler handoff proof | Medium |
| missing access off-work posture integration | Access docs only | off-shift risky actions may be mishandled | build Scheduler/HR truth -> Access posture proof | High |
| missing payroll timesheet handoff | no time facts | payroll correctness risk | build Timesheet -> Payroll boundary | High |
| missing PH1.REM schedule reminder integration | reminder generic only | shift reminders may invent source truth | add source-backed reminder handoff | Medium |
| missing PH1.BCAST roster notification integration | docs only | roster notifications not proven | add BCAST/DELIVERY handoff proof | Medium |
| missing PH1.WRITE schedule guidance boundary | hardcoded calendar text | inconsistent/unsafe user explanations | add PH1.WRITE schedule wording boundary | Medium |
| missing PH1.D/PH1.N schedule proposal path | PH1.N calendar only | messy roster requests not understood | add proposal/extraction shell | Medium |
| missing audit | no workforce audit events | no proof for protected workforce changes | add PH1.J audit evidence pack | High |
| missing SQL persistence | no schedule/roster/timesheet tables | no durable workforce truth | add SQL migration plan | High |
| missing Desktop/iPhone render-only proof | no workforce UI | future UI may overreach | build render-only acceptance | Medium |
| missing Adapter transport-only proof | adapter heuristics exist | adapter may become semantic authority | retire/contain shortcuts | High |
| missing JD live acceptance | no live scheduler proof | product claims unsafe | JD live acceptance pack | High |

## 27. Recommended Future Build Slices

Based on repo truth, future slices should be derived only after Grand Architecture Reconciliation and repo-truth activation:

1. Scheduler/Roster Repo-Truth Activation Pack.
2. PH1.SCHED Naming / Boundary Reconciliation.
3. Scheduler/Roster Contract / State Machine Normalization.
4. Work Pattern / Shift / Roster Type Matrix.
5. Roster Group / Schedule Group Model.
6. Shift Lifecycle.
7. Roster Publish / Change / Notification Flow.
8. Employee Availability + Leave Flow.
9. Clock-In / Clock-Out / Attendance Flow.
10. Timesheet Submission / Approval Flow.
11. Contractor Hours / Overrun Tracking.
12. Position-To-Scheduler/Roster Handoff.
13. Onboarding-To-Scheduler/Roster Handoff.
14. Access Off-Work Posture Integration.
15. Reminder Shift/Follow-Up Integration.
16. Broadcast/Delivery Roster Notification Handoff.
17. Payroll/Timesheet Handoff.
18. Finance/Budget Staffing-Cost Handoff.
19. Overtime / Break / Public Holiday Compliance Boundary.
20. Timezone / Locale / DST Policy.
21. PH1.D + PH1.N Scheduler Proposal Shell.
22. PH1.X Scheduler Route/Risk Validation.
23. PH1.WRITE Scheduler Guidance Boundary.
24. Scheduler/Roster Audit Evidence Pack.
25. SQL Persistence / Migration Plan.
26. Desktop/iPhone Render-Only Scheduler/Roster Proof.
27. Adapter Transport-Only Scheduler/Roster Proof.
28. Old Compatibility Path Retirement Ledger.
29. JD Live Scheduler/Roster Acceptance Pack.

## 28. What Codex Must Not Do

- do not invent scheduler/roster behavior.
- do not create duplicate scheduler/roster engines.
- do not confuse current `PH1.SCHED` WorkOrder retry scheduling with workforce scheduling.
- do not let Position own actual schedules/rosters.
- do not let ONB mutate schedules/rosters directly.
- do not let REM own schedule truth.
- do not let BCAST/DELIVERY own roster truth.
- do not let Access own work availability truth.
- do not let Payroll own raw schedule truth.
- do not let GPT-5.5/OpenAI create active schedules directly.
- do not let Desktop/iPhone decide schedule/roster truth.
- do not let Adapter decide schedule/roster truth.
- do not claim labor/payroll/public holiday compliance without source/owner proof.
- do not use geolocation without consent/policy.
- do not treat calendar draft reminder creation as external calendar execution.
- do not treat `PositionScheduleType::Shift` as shift assignment truth.
- do not treat adapter roster/leave heuristic labels as canonical Scheduler/Roster routing.
- do not delete old paths before proof.
- do not implement from this extraction document alone.

## 29. Final Extracted Architecture Sentence

Selene Scheduler / Roster / Workload is the governed work-time and work-allocation truth boundary: it may own schedules, rosters, shifts, availability, leave, attendance, timesheets, contractor hours, and workload facts where repo truth supports it, while the current `PH1.SCHED` remains an operational WorkOrder retry/wait/fail scheduler, Position owns job requirements, Access owns permissions, Onboarding owns setup, REM owns reminder timing, Broadcast/Delivery owns notifications, Payroll/HR/Finance own pay and money truth, and PH1.D/PH1.N/PH1.WRITE may only assist with understanding and presentation.
