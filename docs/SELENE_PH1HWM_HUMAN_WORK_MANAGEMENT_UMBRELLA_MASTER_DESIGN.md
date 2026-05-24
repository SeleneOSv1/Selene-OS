# Selene PH1.HWM — Human Work Management Umbrella Master Design

MASTER_DESIGN
NOT_RUNTIME_IMPLEMENTATION
PENDING_GRAND_ARCHITECTURE_RECONCILIATION

## 0. Authority and Scope

AGENTS.md controls execution.

This is a docs-only master design. No runtime code was changed. No schema, packet, adapter, client, provider, Access, Payroll/HR, Finance/Budget, Reminder, Broadcast/Delivery, PH1.D, PH1.N, PH1.X, PH1.WRITE, PH1.TASK, PH1.ROSTER, PH1.ATTENDANCE, PH1.WORKLOAD, or PH1.HWM implementation is authorized by this document.

This document preserves and normalizes the Human Work Management vision into PH1.HWM, Selene's umbrella human work coordination stack.

The Scheduler/Roster/Workload repo-truth extraction remains the factual base. Repo truth currently shows `PH1.SCHED` is an internal WorkOrder retry/wait/fail scheduler, not the human workforce scheduler described here. This document therefore uses `PH1.HWM.SCHEDULE` for future human task/non-roster schedule coordination pending Grand Architecture Reconciliation.

Future implementation requires explicit build instruction, approved file scope, repo-truth activation, tests, backend evidence, Access/Authority/Simulation proof where protected actions are involved, PH1.WRITE validation proof, audit proof, and JD live acceptance where visible.

## 1. Executive Target

Selene must become a human work coordination system, not merely a task list, calendar, reminder app, roster app, or ticketing system.

Selene must coordinate human commitments from instruction to outcome across business teams, departments, contractors, households, families, and personal life administration.

Selene must:

- discuss like a human,
- recommend like an expert,
- execute protected actions only through authority and simulation,
- follow through until work is truly complete.

PH1.HWM is the umbrella stack that lets Selene understand work, coordinate people, protect workload, manage follow-through, reason about staffing, and preserve distinct truth owners behind a single human conversation.

Selene may use GPT-5.5 / PH1.D / PH1.N / PH1.X / PH1.WRITE to understand messy requests and present human-quality guidance. Protected execution remains deterministic, authority-gated, simulation-gated, confirmed where required, and audited.

## 2. Correct Split

Task/Schedule Coordination owns:

- tasks,
- due dates,
- non-roster commitments,
- personal/household schedules,
- acceptance,
- negotiation,
- progress,
- completion,
- verification.

Roster/Work-Time Operations owns:

- shifts,
- rosters,
- attendance,
- leave/sick/off-shift truth,
- clock-in/out,
- timesheets,
- coverage,
- overtime,
- labour demand,
- staffing economics.

PH1.HWM coordinates both domains.

Task/schedule is not roster.

Roster is not task/schedule.

They exchange evidence but neither becomes the other.

The separation is intentional because task commitment management and workforce time operations answer different questions, carry different risk, require different state machines, and need different protected-action gates.

Task/schedule asks:

- what needs to be done,
- who asked for it,
- who should do it,
- when it is due,
- whether the receiver accepted,
- whether progress is at risk,
- whether the result is complete and verified.

Roster/work-time asks:

- who is rostered to work,
- when and where they work,
- what role is covered,
- who is absent,
- who can cover,
- who clocked in,
- whether overtime or undercoverage is rising,
- whether labour demand and staffing economics are healthy.

## 3. Naming Law

Current repo `PH1.SCHED` is WorkOrder retry/wait/fail scheduler.

Human-facing schedule coordination must be called `PH1.HWM.SCHEDULE` or another future-approved name.

Do not rename, overload, replace, or reinterpret current `PH1.SCHED` in this docs task.

Do not use "Roaster" for workforce scheduling. The correct term is Roster.

Use canonical task person fields:

- `originator_id` for the person who issued or owns the task request,
- `receiver_id` for the person assigned to complete the task,
- `verifier_id` for the person or authorized role confirming the result where required.

Avoid competing ungoverned field names such as sender, issuer, requester, or assignee unless they are explicit aliases mapped to the canonical fields.

## 4. Domain Map

PH1.HWM coordinates the following future domains while preserving separate truth ownership:

| Domain | Truth Surface | Boundary |
| --- | --- | --- |
| `PH1.TASK` | task truth, assignment, originator, receiver, verifier, progress, blocker, completion, verification | does not own roster publication, attendance, reminder timing, or delivery |
| `PH1.HWM.SCHEDULE` | task due dates, task timelines, non-roster commitments, personal/household schedules | does not own workforce shifts, clock-in/out, overtime, or roster publication |
| `PH1.ROSTER` | workforce roster, shifts, work-time blocks, role/location coverage, shift swaps, absence cover | does not own task completion or actual attendance |
| `PH1.ATTENDANCE` | clock-in/out, breaks, actual worked hours, lateness, no-show, timesheet evidence, contractor hours | does not own scheduled roster truth or payroll calculation |
| `PH1.WORKLOAD` | capacity scoring, overload scoring, skill fit, assignment fit, task and roster pressure | does not assign tasks or publish rosters directly |
| `PH1.HWM.AWARENESS` | operational risk monitoring, overdue/blocked/missing progress/staffing gap signals | does not send messages or mutate truth directly |
| `PH1.HWM.NEGOTIATION` | structured options for deadline, scope, helper, reassignment, dependency, and schedule changes | does not finalize protected outcomes directly |
| `PH1.HWM.STAFFING` | labour demand, under/overstaffing, overtime pressure, skill gaps, headcount/resource recommendations | does not hire, terminate, publish rosters, or own money truth |
| `PH1.HWM.PERFORMANCE` | performance evidence, reliability trends, reward evidence, warning recommendation evidence | does not discipline, pay, reward, or terminate |
| `PH1.HWM.HOUSEHOLD` | household-mode task/schedule adaptation | avoids corporate roster/performance language unless user asks |
| `PH1.HWM.CONTRACTOR` | contractor coordination, hours/evidence/overrun signals, contract follow-through | does not own contracts, AP payments, or contractor commercial terms |

Cross-domain owners remain separate:

- PH1.REM owns reminder timing.
- PH1.BCAST / PH1.DELIVERY owns notification and delivery.
- PH1.WRITE owns final user-facing wording.
- Access/Governance owns permissions and protected access gates.
- Payroll/HR owns pay, employment, leave classification where applicable, and employment-status truth.
- Compensation owns bonus/reward execution and compensation decisions where present.
- Finance/Budget owns budget, spend, cost, margin, and money truth.
- PH1.D / GPT-5.5 may propose.
- PH1.N may extract.
- PH1.X may validate route/risk.
- Deterministic owners execute only after required gates pass.

## 5. Shared Operating Model

The shared PH1.HWM operating flow is:

user voice/type
-> PH1.D / GPT-5.5 proposal
-> PH1.N extraction
-> PH1.X route/risk validation
-> Access/Authority/Simulation where needed
-> correct HWM owner
-> PH1.WRITE final wording
-> PH1.REM reminder timing
-> PH1.BCAST/DELIVERY notification
-> audit

The Human Understanding layer may handle messy language, accents, fragments, mixed language, incomplete ideas, emotional tone, and unclear references.

The deterministic validation layer must evaluate:

- required fields,
- ambiguity,
- task or roster class,
- risk class,
- authority class,
- simulation match,
- policy constraints,
- protected-action status,
- available evidence,
- audit readiness.

Provider confidence is proposal evidence only. It never grants authority and never replaces deterministic validation.

Personality and communication style may affect tone, pacing, warmth, assertiveness, corporate vs household language, and spoken/display wording. Personality must never alter authority rules, simulation rules, confirmation requirements, audit requirements, or protected-action boundaries.

## 6. Preservation Ledger

| Original HWM Concept / Section | Preserved In | Treatment | Notes |
| --- | --- | --- | --- |
| Complete Human Work Management vision | Umbrella | Preserved | PH1.HWM remains the coordination stack for enterprise, teams, contractors, households, and personal administration. |
| Deterministic + probabilistic coordination system | Umbrella | Preserved | GPT-5.5 / PH1.D / PH1.N may assist; protected execution remains deterministic. |
| "Selene discusses like a human" operating standard | Umbrella | Preserved | Routed through shared operating model and PH1.WRITE. |
| "Selene recommends like an expert" operating standard | Umbrella, Task/Schedule, Roster/Attendance/Staffing, Workload/Awareness/Negotiation/Performance | Preserved | Recommendations are domain-owned and evidence-backed. |
| "Selene executes protected actions only through authority and simulation" | Umbrella | Preserved | Protected action safety is explicit. |
| Task/schedule versus roster/work-time split | Umbrella | Preserved and normalized | Split is now architecture law for this design set. |
| Originator, Receiver, Verifier naming | Task/Schedule | Preserved | Canonical fields are `originator_id`, `receiver_id`, and `verifier_id`. |
| Roster naming law and "Roaster" rejection | Umbrella, Roster/Attendance/Staffing | Preserved | Workforce scheduling term is Roster. |
| Protected action versus coordination action | Umbrella | Preserved | Protected actions require simulation, authority, confirmation where required, and audit. |
| Human chaos / messy language understanding | Umbrella | Preserved | PH1.D / PH1.N / PH1.WRITE assist; no protected execution authority. |
| Provider output proposal contract | Umbrella | Preserved | Provider outputs are candidates consumed by Selene validators. |
| PH1.TASK task truth | Task/Schedule | Preserved | Task truth moved to dedicated task/commitment document. |
| PH1.HWM.SCHEDULE task/non-roster schedule truth | Task/Schedule | Preserved and renamed | Uses future-safe name to avoid current `PH1.SCHED` collision. |
| PH1.ROSTER workforce roster truth | Roster/Attendance/Staffing | Preserved | Owns shifts, coverage, swaps, and roster publication surface. |
| PH1.ATTENDANCE actual presence truth | Roster/Attendance/Staffing | Preserved | Owns clock-in/out, breaks, timesheets, contractor hours, payroll evidence handoff. |
| PH1.WORKLOAD capacity truth | Workload/Awareness/Negotiation/Performance | Preserved | Shared capacity engine across task and roster domains. |
| PH1.REM reminder timing | Umbrella, Task/Schedule, Roster/Attendance/Staffing | Preserved | Reminder timing stays separate from task/roster truth. |
| PH1.BCAST / PH1.DELIVERY notification delivery | Umbrella, Task/Schedule, Roster/Attendance/Staffing | Preserved | Delivery stays separate from truth and timing. |
| PH1.HWM.AWARENESS risk monitoring | Workload/Awareness/Negotiation/Performance | Preserved | Detects risk, does not execute protected action. |
| PH1.HWM.NEGOTIATION structured negotiation | Workload/Awareness/Negotiation/Performance | Preserved | Creates options; truth owners record approved results. |
| PH1.HWM.STAFFING staffing economics | Roster/Attendance/Staffing | Preserved | Labour demand, resource planning, position-cost recommendations, employee/contractor comparison. |
| PH1.HWM.PERFORMANCE performance evidence | Workload/Awareness/Negotiation/Performance | Preserved | Advisory evidence only, not automatic discipline. |
| PH1.HWM.HOUSEHOLD household mode | Umbrella, Task/Schedule | Preserved | Lightweight, natural, non-corporate task/schedule coordination. |
| PH1.HWM.CONTRACTOR contractor coordination | Umbrella, Roster/Attendance/Staffing, Workload/Awareness/Negotiation/Performance | Preserved | Contractor hours, overrun, deliverables, expiry/reminder handoffs. |
| Business critical task lifecycle | Task/Schedule | Preserved | Full lifecycle retained with confirmation, lock, risk, verification, rework, archive. |
| Normal business task lifecycle | Task/Schedule | Preserved | Simpler accountability lifecycle retained. |
| Household task lifecycle | Task/Schedule | Preserved | Lightweight household lifecycle retained. |
| Personal reminder lifecycle | Task/Schedule and PH1.REM boundary | Preserved | Reminder timing remains PH1.REM-owned. |
| Closed-loop task management | Task/Schedule | Preserved | Task is not complete until required completion, evidence, verification, and outcome reporting pass. |
| Task schedule coordination | Task/Schedule | Preserved | Due dates, start windows, check-ins, recurring commitments, conflicts. |
| Task follow-up and reminder control | Task/Schedule, Umbrella | Preserved | PH1.REM timing and PH1.BCAST delivery boundaries retained. |
| Originator reporting law | Task/Schedule | Preserved | Originator must not be left blind. |
| Failure reason capture | Task/Schedule, Workload/Awareness/Negotiation/Performance | Preserved | Reason classification separates worker, planning, dependency, workload, resource, and management causes. |
| Intelligent task assignment flow | Task/Schedule, Workload/Awareness/Negotiation/Performance | Preserved | Smart allocation now consumes capacity and roster evidence without mutating roster. |
| Task transfer and reassignment control | Task/Schedule, Workload/Awareness/Negotiation/Performance | Preserved | Transfer requires reason, alternative candidates, approval where required, and audit. |
| Receiver negotiation flow | Task/Schedule, Workload/Awareness/Negotiation/Performance | Preserved | Accept, adjust, decline, clarify, help, block, deadline/scope/reassignment requests retained. |
| Task rescue before discipline | Task/Schedule, Workload/Awareness/Negotiation/Performance | Preserved | Support-first doctrine retained. |
| Roster scope | Roster/Attendance/Staffing | Preserved | Draft/published rosters, shifts, coverage, swaps, breaks, acknowledgement. |
| Sick day, absence, cover, swap, owed-shift flow | Roster/Attendance/Staffing | Preserved | Replacement ranking and manager escalation boundaries retained. |
| Attendance scope | Roster/Attendance/Staffing | Preserved | Actual presence truth separated from roster. |
| Leave, absence, payroll evidence boundary | Roster/Attendance/Staffing | Preserved | HWM supplies evidence; Payroll/HR calculates/executes. |
| Location-verified attendance and privacy boundary | Roster/Attendance/Staffing | Preserved | Policy, notice, consent where required, access, audit, retention, manual review. |
| Overtime detection and approval flow | Roster/Attendance/Staffing | Preserved | Projected/actual overtime, approval, payroll evidence. |
| Roster demand forecasting | Roster/Attendance/Staffing | Preserved | Busy/slow periods, headcount, skill, budget, overtime, compliance constraints. |
| Automated roster recommendation and publication boundary | Roster/Attendance/Staffing | Preserved | Drafts/recommendations allowed; publication protected. |
| Overtime and labour cost control | Roster/Attendance/Staffing | Preserved | Cost-aware staffing retained. |
| Overstaffing and understaffing detection | Roster/Attendance/Staffing | Preserved | Separates headcount, skill mix, scheduling, and performance problems. |
| Business staffing intelligence | Roster/Attendance/Staffing | Preserved | Management insight and headcount review recommendations retained. |
| Resource planning, position economics, forecast return | Roster/Attendance/Staffing | Preserved | Position-cost and employee/contractor cost comparisons retained as handoffs. |
| Workload across both domains | Workload/Awareness/Negotiation/Performance | Preserved | Consumes task, schedule, roster, attendance, leave, skill, reliability, and overtime evidence. |
| Performance intelligence and scorecards | Workload/Awareness/Negotiation/Performance | Preserved | Fairness adjustments and advisory-only status retained. |
| Performance-to-reward and bonus handoff | Workload/Awareness/Negotiation/Performance | Preserved | Reward evidence retained; Compensation/Payroll execute. |
| Poor performance and company notice workflow | Workload/Awareness/Negotiation/Performance | Preserved | Evidence/support/HR review path retained; no automatic termination. |
| Cross-domain task assignment flow | Umbrella, Task/Schedule, Workload/Awareness/Negotiation/Performance | Preserved | HWM owner routing normalized. |
| Cross-domain roster publication flow | Umbrella, Roster/Attendance/Staffing | Preserved | Roster draft, staffing, workload, approval, delivery, reminders, attendance retained. |
| Falling-behind task flow | Task/Schedule, Workload/Awareness/Negotiation/Performance | Preserved | Awareness, reminders, blocker capture, originator updates retained. |
| Roster impact on tasks | Workload/Awareness/Negotiation/Performance | Preserved | Roster evidence informs capacity. |
| Task impact on roster | Roster/Attendance/Staffing | Preserved | Task demand can inform staffing recommendations without task owning roster. |
| Absence cover flow | Roster/Attendance/Staffing | Preserved | Cover search, candidate ranking, delivery, approved update, payroll evidence handoff retained. |
| Payroll and reward evidence handoff | Roster/Attendance/Staffing, Workload/Awareness/Negotiation/Performance | Preserved | HWM provides evidence; Payroll/Compensation execute. |
| Discuss, diagnose, recommend, decide, action, follow-through, close model | Umbrella | Preserved | Defines Selene's operations-manager experience. |
| Business mode | Umbrella, Task/Schedule, Roster/Attendance/Staffing | Preserved | Authority, audit, escalation, accountability retained. |
| Household mode | Umbrella, Task/Schedule | Preserved | Warm non-corporate handling retained. |
| Contractor coordination | Umbrella, Roster/Attendance/Staffing, Workload/Awareness/Negotiation/Performance | Preserved | Contractor hours, deliverables, invoice/contract handoff, overrun retained. |
| Fairness, audit, human review | Umbrella, Workload/Awareness/Negotiation/Performance | Preserved | Explainability and human review retained. |
| Protected action safety rules | Umbrella | Preserved | Protected action list retained and normalized. |
| Task & Schedule canonical packets | Task/Schedule | Preserved | TaskPacket, TaskSchedulePacket, TaskFollowUpPacket, TaskOutcomePacket, NegotiationPacket, TaskTransferPacket, OriginatorReportPacket. |
| Roster & Work-Time canonical packets | Roster/Attendance/Staffing | Preserved | RosterPacket, RosterDemandPacket, AttendancePacket, StaffingEconomicsPacket, ShiftCoverPacket, LeaveAbsencePacket, LocationAttendancePacket, OvertimeApprovalPacket, PayrollEvidencePacket, ResourceForecastPacket, PositionEconomicsPacket. |
| Shared HWM packets | Workload/Awareness/Negotiation/Performance | Preserved | WorkloadPacket, SkillCapacityPacket, ReminderPlanPacket, ProgressRiskPacket, PerformancePacket, PerformanceNoticePacket, RewardEligibilityPacket. |
| Task simulations | Task/Schedule | Preserved | Task lifecycle and follow-up simulations retained. |
| Schedule simulations | Task/Schedule | Preserved | Non-roster task schedule simulations retained. |
| Roster simulations | Roster/Attendance/Staffing | Preserved | Roster, cover, swap, optimization simulations retained. |
| Attendance simulations | Roster/Attendance/Staffing | Preserved | Clock-in/out, timesheet, location, overtime, payroll evidence simulations retained. |
| Staffing simulations | Roster/Attendance/Staffing | Preserved | Forecast, cost, contractor comparison simulations retained. |
| Workload simulations | Workload/Awareness/Negotiation/Performance | Preserved | Capacity, ranking, overload, rebalance simulations retained. |
| Performance simulations | Workload/Awareness/Negotiation/Performance | Preserved | Scorecard, support, notices, improvement, reward simulations retained. |
| Reminder simulations | Umbrella and domain boundaries | Preserved | PH1.REM owns reminder timing. |
| Broadcast simulations | Umbrella and domain boundaries | Preserved | PH1.BCAST / PH1.DELIVERY owns delivery. |
| End-to-end task allocation example | Task/Schedule, Workload/Awareness/Negotiation/Performance | Preserved | Drawings by Friday example retained. |
| Task falling behind example | Task/Schedule | Preserved | Progress check and blocker/originator update retained. |
| Transfer request example | Task/Schedule, Workload/Awareness/Negotiation/Performance | Preserved | Alternative worker options retained. |
| Roster optimization example | Roster/Attendance/Staffing | Preserved | Friday demand and Monday overstaffing retained. |
| Sick day and shift replacement example | Roster/Attendance/Staffing | Preserved | Same-day cover flow retained. |
| Overtime approval example | Roster/Attendance/Staffing | Preserved | Projected overtime approval retained. |
| Resource planning and position cost example | Roster/Attendance/Staffing | Preserved | Packaging worker cost/return example retained. |
| Schedule versus roster distinction examples | Task/Schedule, Roster/Attendance/Staffing | Preserved | Drawing deadline vs Friday shift examples retained. |
| Poor performance pattern example | Workload/Awareness/Negotiation/Performance | Preserved | Manager review only, no automatic termination. |
| Household coordination example | Task/Schedule | Preserved | Dog feeding reminder retained. |
| Final HWM operating standard | Umbrella | Preserved | Split into umbrella plus three focused documents. |

## 7. Protected Action Safety

Protected actions include:

- roster publication,
- shift enforcement,
- leave approval/denial,
- timesheet approval,
- overtime approval,
- formal warnings,
- employment status changes,
- payroll actions,
- pay changes,
- contractor commercial changes,
- access changes,
- authority changes,
- formal HR notices,
- location-tracking policy changes.

Selene may recommend and prepare.

Protected execution requires simulation, authority, confirmation where required, and audit.

No Simulation -> No Execution.

PH1.HWM may coordinate options, collect evidence, ask clarifying questions, recommend action, draft wording, and follow up. It must not bypass Access, Authority, Simulation, HR, Payroll, Finance/Budget, Compensation, legal/compliance owners, PH1.WRITE, PH1.REM, or PH1.BCAST / PH1.DELIVERY boundaries.

## 8. Final Umbrella Sentence

PH1.HWM is Selene’s umbrella human work coordination stack: it coordinates tasks, schedules, workload, rosters, attendance, staffing, negotiation, performance evidence, household duties, contractors, reminders, and notifications while preserving separate truth owners and protected execution gates.
