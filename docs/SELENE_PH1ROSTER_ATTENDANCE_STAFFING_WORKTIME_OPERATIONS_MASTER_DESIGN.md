# Selene PH1.ROSTER + PH1.ATTENDANCE + PH1.HWM.STAFFING — Workforce Time Operations Master Design

MASTER_DESIGN
NOT_RUNTIME_IMPLEMENTATION
PENDING_GRAND_ARCHITECTURE_RECONCILIATION

## 0. Authority and Scope

AGENTS.md controls execution.

This is a docs-only master design. No runtime code was changed. This document does not authorize implementation.

This document owns the workforce time side of the PH1.HWM split: rosters, shifts, work-time blocks, attendance, leave/absence cover, clock-in/out, timesheets, coverage, overtime, labour demand, staffing economics, and payroll evidence handoff.

The Scheduler/Roster/Workload repo-truth extraction remains the factual base. Current repo `PH1.SCHED` is a WorkOrder retry/wait/fail scheduler, not this future workforce roster owner.

Future implementation requires repo-truth activation, approved file scope, tests, backend evidence, Access/Authority/Simulation proof where protected effects exist, Payroll/HR/Finance/Compensation owner proof where applicable, PH1.WRITE validation proof, audit proof, and JD live acceptance where visible.

## 1. Executive Target

Roster/Attendance/Staffing manages:

- who is working,
- when they work,
- where they work,
- what role they cover,
- who is absent,
- who can cover,
- who clocked in,
- who is late,
- overtime risk,
- coverage risk,
- staffing economics,
- labour demand.

This is workforce time and coverage management. It is not task truth, task verification, personal household scheduling, payroll calculation, or finance truth.

Selene must coordinate workforce time like a calm operations manager: prepare rosters, detect cover gaps, handle sick-day replacement where policy allows, track attendance evidence, reduce unnecessary overtime, and route protected decisions through authority and simulation.

## 2. Owner Split

PH1.ROSTER owns:

- roster truth,
- shift truth,
- work-time blocks,
- coverage,
- role/location coverage,
- shift swaps,
- absence cover,
- roster acknowledgement.

PH1.ATTENDANCE owns:

- clock-in,
- clock-out,
- breaks,
- actual worked hours,
- lateness,
- no-show,
- timesheet submission,
- timesheet approval evidence,
- contractor hours,
- approved overtime evidence,
- payroll evidence handoff.

PH1.HWM.STAFFING owns:

- demand forecasting,
- under/overstaffing,
- labour budget risk,
- overtime cost pressure,
- skill gaps,
- headcount recommendations,
- employee vs contractor cost comparison.

Payroll/HR owns pay and employment truth.

Finance/Budget owns money truth.

Compensation owns reward/bonus execution where present.

Access owns permissions.

PH1.REM owns reminders.

PH1.BCAST / PH1.DELIVERY owns notifications.

PH1.WRITE owns final user-facing wording.

PH1.TASK owns task truth. PH1.HWM.SCHEDULE owns non-roster task and personal/household schedule coordination. Roster/Attendance must not become task management.

## 3. Roster Lifecycle

Required future roster lifecycle:

- Draft,
- Pending Review,
- Published,
- Changed,
- Locked,
- Cancelled,
- Archived.

Draft roster creation may be optimized by Selene using staffing, workload, leave, and demand evidence.

Roster publication is protected where company policy, workforce impact, legal/compliance posture, or authority rules require it. Selene may prepare and recommend, but publication must pass required simulation, authority, confirmation, and audit gates.

Existing employees/users are not silently moved, reduced, or assigned without the required policy and authority posture.

## 4. Shift Lifecycle

Required future shift lifecycle:

- Open,
- Assigned,
- Accepted,
- Declined,
- Swapped,
- Covered,
- Completed,
- Missed,
- Cancelled.

Shift lifecycle must record:

- worker/person,
- role covered,
- location,
- required skills/certifications,
- start/end time,
- timezone,
- break expectations,
- acknowledgement state,
- cover state,
- overtime risk,
- leave/absence conflict,
- audit reference.

## 5. Leave / Absence / Sick Day Cover

Required flow:

worker reports absence
-> classify absence
-> identify affected shift
-> check role/skill/location
-> check coverage risk
-> rank replacements
-> contact candidates
-> confirm replacement
-> update roster through approved simulation
-> notify worker/replacement/supervisor
-> hand leave/payroll evidence to correct owners

Selene must support:

- same-day sick-day replacement,
- emergency absence cover,
- partial-shift cover,
- full-shift replacement,
- shift swap,
- owed-shift arrangement where company policy allows,
- cover escalation when no replacement accepts,
- supervisor approval where required.

Replacement ranking may consider:

- skill,
- availability,
- location,
- seniority/certification,
- fatigue,
- overtime risk,
- cost,
- reliability,
- fairness,
- policy eligibility.

Supervisor involvement should happen only when approval, overtime, compliance, policy, judgment, or failed cover requires escalation.

## 6. Attendance + Timesheet

PH1.ATTENDANCE must own:

- clock-in,
- clock-out,
- breaks,
- late/no-show,
- timesheet submission,
- manager approval,
- payroll evidence handoff,
- location validation where lawful/policy-approved.

Attendance is actual presence truth.

Roster is scheduled work-time truth.

Task schedule is commitment timing truth.

Attendance may produce payroll evidence, but Payroll calculates and pays.

## 7. Overtime Approval

Selene must detect:

- projected overtime,
- actual overtime,
- overtime reason,
- applicable policy,
- approval required,
- manager notification,
- approved vs unapproved overtime,
- payroll evidence handoff.

Example:

"Michael is projected to exceed his approved shift by 45 minutes to complete dispatch. Overtime approval is required. I recommend approving 1 hour because replacement cover would cost more and delay dispatch. Do you approve?"

Overtime approval is protected where company policy requires approval.

Repeated overtime should also feed staffing and workload analysis so Selene can distinguish short-term operational need from structural understaffing.

## 8. Location-Verified Attendance

Location-verified attendance is allowed only with:

- policy,
- lawful notice,
- consent where required,
- role-based access,
- audit,
- retention limits,
- manual review.

Allowed methods may include:

- GPS,
- Wi-Fi,
- Bluetooth,
- QR code,
- kiosk,
- device check-in,
- supervisor confirmation,
- photo proof where lawful.

Location evidence is not a surveillance license.

Selene must not secretly track employees. Location validation must be minimum necessary, policy-bound, privacy-protected, auditable, and reviewable.

## 9. Roster Demand Forecasting

PH1.HWM.STAFFING must help Selene avoid blind rosters.

Inputs may include:

- sales history,
- bookings,
- production volume,
- delivery schedule,
- customer demand,
- seasonality,
- holidays,
- events,
- weather where relevant,
- availability,
- leave,
- skills,
- staffing rules,
- budget limits.

Outputs may include:

- busy periods,
- slow periods,
- required headcount,
- required skills,
- overstaffing,
- understaffing,
- overtime risk,
- labour budget risk.

Selene may prepare optimized roster drafts and staffing recommendations. It may not publish or enforce protected roster changes without required gates.

## 10. Staffing Economics

PH1.HWM.STAFFING must reason about:

- idle labour,
- overtime cost,
- contractor cost,
- skill gaps,
- labour waste,
- headcount review,
- resource forecast,
- position cost estimate,
- employee vs contractor comparison,
- forecast return.

Selene should separate these problems:

- not enough people,
- enough people but wrong skills,
- enough skills but poor scheduling,
- enough staff but poor performance.

Those problems require different recommendations.

PH1.HWM.STAFFING may recommend headcount review, internal reallocation, contractor use, hiring analysis, or reduced roster waste. It does not hire, terminate, reduce employment terms, or own money truth.

## 11. Payroll / Reward / Finance Handoff

HWM provides evidence:

- scheduled hours,
- actual hours,
- overtime,
- leave type,
- public holiday hours,
- contractor hours,
- attendance reliability,
- bonus evidence.

Payroll/Compensation/Finance execute their own truth.

Payroll owns pay calculation and salary processing.

Compensation owns approved rewards/bonus execution where present.

Finance/Budget owns financial impact, budget availability, margin, spend, and money truth.

HWM must provide clean evidence and never silently take over those owner surfaces.

## 12. Required Logical Packets

These are future logical packets. This document does not claim they currently exist in repo truth.

### RosterPacket

- `roster_id`
- `shift_start`
- `shift_end`
- `role`
- `location`
- `assigned_person_id`
- `overtime_risk`
- `leave_conflict`
- `published_status`
- `acknowledgement_status`
- `cover_status`
- `replacement_required`

### RosterDemandPacket

- `period_start`
- `period_end`
- `location`
- `department`
- `forecast_demand_level`
- `required_headcount`
- `required_skills`
- `current_roster_count`
- `shortage_count`
- `surplus_count`
- `overtime_risk`
- `labour_budget_risk`
- `recommendation`

### AttendancePacket

- `attendance_id`
- `person_id`
- `roster_id`
- `clock_in_at`
- `clock_out_at`
- `break_periods`
- `actual_hours`
- `lateness_status`
- `no_show_status`
- `timesheet_status`
- `approval_status`
- `location_validation_status`
- `overtime_status`

### StaffingEconomicsPacket

- `department_id`
- `labour_cost`
- `revenue_or_output_signal`
- `idle_hours`
- `overtime_hours`
- `contractor_hours`
- `understaffed_periods`
- `overstaffed_periods`
- `skill_gap_count`
- `recommended_adjustments`

### ShiftCoverPacket

- `cover_request_id`
- `roster_id`
- `absent_person_id`
- `absence_type`
- `affected_shift_start`
- `affected_shift_end`
- `required_role`
- `required_skills`
- `coverage_risk`
- `candidate_rankings`
- `selected_replacement_id`
- `approval_required`
- `approval_status`
- `cover_status`
- `audit_ref`

### LeaveAbsencePacket

- `absence_id`
- `person_id`
- `absence_type`
- `reported_at`
- `affected_roster_ids`
- `leave_category`
- `paid_or_unpaid_status`
- `evidence_required`
- `approval_required`
- `approval_status`
- `payroll_handoff_required`
- `audit_ref`

### LocationAttendancePacket

- `location_check_id`
- `person_id`
- `roster_id`
- `expected_location`
- `validation_method`
- `validation_status`
- `checked_at`
- `mismatch_reason`
- `manual_review_required`
- `privacy_policy_ref`
- `retention_policy_ref`
- `audit_ref`

### OvertimeApprovalPacket

- `overtime_request_id`
- `person_id`
- `roster_id`
- `projected_overtime_minutes`
- `actual_overtime_minutes`
- `reason_code`
- `supervisor_id`
- `approval_required`
- `approval_status`
- `payroll_handoff_status`
- `audit_ref`

### PayrollEvidencePacket

- `payroll_evidence_id`
- `person_id`
- `period_start`
- `period_end`
- `scheduled_hours`
- `actual_hours`
- `approved_overtime_hours`
- `leave_hours_by_type`
- `public_holiday_hours`
- `unpaid_absence_hours`
- `contractor_hours`
- `location_validation_summary`
- `payroll_owner`
- `handoff_status`
- `audit_ref`

### ResourceForecastPacket

- `resource_forecast_id`
- `department_id`
- `period_start`
- `period_end`
- `forecast_demand`
- `current_headcount`
- `required_headcount`
- `required_skills`
- `current_labour_cost`
- `forecast_labour_cost`
- `forecast_output_or_revenue`
- `shortage_or_surplus`
- `recommended_action`
- `expected_return_summary`

### PositionEconomicsPacket

- `position_analysis_id`
- `position_title`
- `required_skills`
- `market_salary_estimate`
- `recruitment_cost_estimate`
- `onboarding_cost_estimate`
- `training_cost_estimate`
- `contractor_alternative_cost`
- `overtime_reduction_estimate`
- `productivity_gain_estimate`
- `expected_return_summary`
- `recommendation`

## 13. Required Simulation Inventory

Required future simulations include:

- `ROSTER.CREATE_DRAFT`
- `ROSTER.ASSIGN_SHIFT`
- `ROSTER.PUBLISH`
- `ROSTER.ACCEPT_SHIFT`
- `ROSTER.REPORT_ABSENCE`
- `ROSTER.REQUEST_SHIFT_COVER`
- `ROSTER.FIND_COVER`
- `ROSTER.RANK_REPLACEMENTS`
- `ROSTER.CONFIRM_REPLACEMENT`
- `ROSTER.UPDATE_SHIFT_COVER`
- `ROSTER.REQUEST_SHIFT_SWAP`
- `ROSTER.APPROVE_SHIFT_SWAP`
- `ROSTER.FORECAST_DEMAND`
- `ROSTER.CREATE_OPTIMIZED_DRAFT`
- `ROSTER.DETECT_OVERSTAFFING`
- `ROSTER.DETECT_UNDERSTAFFING`
- `ROSTER.MINIMIZE_OVERTIME`
- `ATTENDANCE.CLOCK_IN`
- `ATTENDANCE.CLOCK_OUT`
- `ATTENDANCE.SUBMIT_TIMESHEET`
- `ATTENDANCE.APPROVE_TIMESHEET`
- `ATTENDANCE.CORRECT_MISSED_CLOCK`
- `ATTENDANCE.VALIDATE_LOCATION`
- `ATTENDANCE.DETECT_OVERTIME_RISK`
- `ATTENDANCE.REQUEST_OVERTIME_APPROVAL`
- `ATTENDANCE.HANDOFF_PAYROLL_EVIDENCE`
- `LEAVE.RECORD_SICK_DAY`
- `LEAVE.CLASSIFY_LEAVE_TYPE`
- `STAFFING.CREATE_RESOURCE_FORECAST`
- `STAFFING.ESTIMATE_POSITION_COST`
- `STAFFING.COMPARE_EMPLOYEE_CONTRACTOR_COST`

Simulation names are logical design targets pending Grand Architecture Reconciliation and repo-truth activation.

## 14. Examples

### Sick Day Cover Request

Worker:

"Today I'm having a sick day. Please arrange a replacement for my shift."

Selene should:

- classify the sick day or absence,
- identify affected shift,
- check required role and skills,
- check minimum coverage,
- find eligible replacements,
- rank replacements by suitability, cost, overtime, fatigue, location, and fairness,
- contact the best candidate,
- confirm acceptance,
- update the roster through approved simulation,
- hand absence and payroll evidence to HR/payroll owners,
- involve the supervisor only if approval or escalation is required.

Expected response:

"I've recorded that you cannot attend today's 2 PM-10 PM shift. Sarah is the best replacement because she has the same role, is available, and will not trigger overtime. I'll request confirmation from Sarah now and escalate to the supervisor only if cover is not confirmed."

### Overtime Approval

"Michael is projected to exceed his approved shift by 45 minutes to complete dispatch. Overtime approval is required. I recommend approving 1 hour because replacement cover would cost more and delay dispatch. Do you approve?"

### Friday High-Demand Roster

"Friday evening demand is forecast high, but the current roster is short by 3 staff. Monday afternoon is forecast slow and overstaffed by 2. Moving two shifts from Monday to Friday reduces overtime risk and improves coverage. Do you want me to prepare the revised roster draft?"

### Overstaffed Monday / Understaffed Thursday

"This department is overstaffed on Mondays and understaffed on Thursdays. There are 42 idle labour hours on Monday, while Thursday generates 18 overtime hours. I recommend moving two shifts from Monday to Thursday."

### Packaging Worker Resource Question

Manager:

"Do we need another packaging worker?"

Expected response:

"Packaging is short by 38 labour hours per week during peak periods. Current overtime costs $4,800 per month. A part-time packaging worker is estimated at $3,200 per month. Hiring one part-time worker may reduce overtime by 70 percent and improve Friday dispatch reliability."

### Employee Versus Contractor Comparison

Selene may compare contractor cost, overtime reduction, skill gap, hiring time, onboarding time, and budget impact, then route pay, contract, and hiring decisions to their owners.

## 15. What Must Not Happen

- no roster truth inside Position,
- no attendance truth inside Roster,
- no payroll calculation inside Attendance,
- no finance truth inside Staffing,
- no location tracking without policy/consent where required,
- no auto-discipline from attendance alone,
- no roster publishing without authority/simulation where required,
- no implementation from this document alone.
