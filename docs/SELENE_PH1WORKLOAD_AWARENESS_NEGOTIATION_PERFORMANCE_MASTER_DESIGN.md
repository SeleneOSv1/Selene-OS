# Selene PH1.WORKLOAD + HWM Awareness / Negotiation / Performance Master Design

MASTER_DESIGN
NOT_RUNTIME_IMPLEMENTATION
PENDING_GRAND_ARCHITECTURE_RECONCILIATION

## 0. Authority and Scope

AGENTS.md controls execution.

This is a docs-only master design. No runtime code was changed. This document does not authorize implementation.

This document owns shared human capacity intelligence across both PH1.TASK / PH1.HWM.SCHEDULE and PH1.ROSTER / PH1.ATTENDANCE / PH1.HWM.STAFFING domains.

It defines the future boundaries for capacity scoring, operational awareness, structured negotiation, performance evidence, reward evidence, support-before-discipline, and cross-domain human workload recommendations pending Grand Architecture Reconciliation.

Future implementation requires repo-truth activation, approved file scope, tests, backend evidence, Access/Authority/Simulation proof where protected effects exist, PH1.WRITE validation proof, HR/Payroll/Compensation owner proof where applicable, audit proof, and JD live acceptance where visible.

## 1. Executive Target

This stack measures capacity, detects risk, negotiates changes, monitors performance evidence, and helps Selene recommend human-operational actions.

The target is:

- protect workers from unfair overload,
- protect originators from silent failure,
- protect managers from invisible risk,
- recommend better assignments,
- rescue failing work before discipline,
- collect fair evidence where performance or reward decisions may later be considered.

This stack does not own task truth, roster truth, reminder timing, notification delivery, HR discipline, pay, bonuses, or employment status.

## 2. Owner Split

PH1.WORKLOAD owns:

- capacity scoring,
- overload scoring,
- available capacity,
- active task burden,
- rostered-hour context,
- skill matching,
- assignment fit ranking.

PH1.HWM.AWARENESS owns:

- overdue detection,
- at-risk task detection,
- blocked dependency detection,
- missing progress,
- staffing gap signal,
- contractor overrun signal,
- burnout signal,
- roster coverage risk.

PH1.HWM.NEGOTIATION owns:

- deadline-change proposals,
- scope-change proposals,
- reassignment proposals,
- helper proposals,
- dependency negotiation,
- receiver adjustment requests.

PH1.HWM.PERFORMANCE owns:

- performance evidence,
- reliability trends,
- scorecards,
- reward eligibility evidence,
- warning recommendation evidence.

PH1.TASK records approved task truth changes.

PH1.HWM.SCHEDULE records approved non-roster schedule changes.

PH1.ROSTER records approved roster/shift changes.

PH1.ATTENDANCE records actual presence and timesheet evidence.

PH1.REM owns reminder timing.

PH1.BCAST / PH1.DELIVERY owns delivery.

PH1.WRITE owns final user-facing wording.

HR, Payroll, Compensation, Finance/Budget, Access, and Authority owners retain their canonical truth surfaces.

## 3. Workload Across Both Domains

Workload must consume:

- active tasks,
- due dates,
- estimated effort,
- rostered hours,
- actual hours,
- leave,
- sick days,
- meetings,
- travel time,
- timezone,
- overtime risk,
- required skill,
- historical reliability.

Workload must calculate whether a person can reasonably take more work. It should rank candidate assignees by:

- skill match,
- availability,
- workload,
- reliability,
- cost where relevant,
- overtime impact,
- location,
- urgency fit,
- past success on similar work,
- compliance eligibility.

Example:

"Sarah has the required CAD skill, but she is already overloaded with 4 urgent tasks due this week. Michael has the same CAD skill, less workload, and can complete it before Friday without overtime. I recommend Michael."

PH1.WORKLOAD recommends and scores. It does not assign tasks directly.

## 4. Awareness / Risk Monitoring

PH1.HWM.AWARENESS must detect:

- overdue tasks,
- ignored tasks,
- blocked tasks,
- overloaded workers,
- staffing gaps,
- repeated delays,
- contractor overrun,
- missing progress,
- low response,
- project slippage,
- roster undercoverage.

Awareness may trigger recommendations, reminders, escalations, or negotiation needs.

It does not execute protected action.

Awareness must not send messages directly. It should request PH1.REM reminder timing, PH1.BCAST / PH1.DELIVERY delivery, PH1.WRITE wording, and the correct truth owner for any actual state change.

## 5. Negotiation Engine

PH1.HWM.NEGOTIATION owns structured negotiation:

- deadline change,
- scope reduction,
- helper assignment,
- reassignment,
- dependency resolution,
- schedule change proposal,
- receiver counterproposal.

Negotiation creates options.

Authorized humans or simulations decide protected outcomes.

Negotiation should capture:

- who proposed the change,
- current value,
- requested value,
- reason code,
- deadline impact,
- workload impact,
- roster impact where relevant,
- originator approval requirement,
- approval state,
- audit reference.

PH1.HWM.NEGOTIATION does not mutate task, schedule, roster, access, HR, payroll, or finance truth directly.

## 6. Performance Evidence

PH1.HWM.PERFORMANCE may measure:

- on-time completion,
- late completion,
- missed tasks,
- accepted task completion,
- ignored tasks,
- blocker reporting,
- response time,
- rework,
- quality verification,
- escalation frequency,
- improvement trend,
- attendance reliability where lawful,
- task rescue contribution.

Performance must be fair.

Adjust for:

- difficulty,
- workload,
- skill fit,
- leave/sickness,
- dependency blocks,
- missing info,
- unrealistic deadline,
- management delay.

Performance scores are advisory evidence. They are not automatic discipline, pay, reward, termination, or employment action.

No performance score may be used for formal HR action unless failure reason classification, support-before-discipline checks, evidence review, authorization, policy, and audit requirements are satisfied.

## 7. Support Before Discipline

Before formal poor-performance action:

- check workload,
- check clarity,
- check resources,
- check support provided,
- classify reason,
- offer rescue,
- prepare manager review only after evidence.

Selene must not automatically punish.

Support options may include:

- clarify the task,
- provide a checklist,
- request missing information,
- assign helper,
- split task,
- transfer task,
- extend deadline,
- recommend training,
- remove lower-priority work,
- resolve dependency.

Only after reasonable support and fair evidence may Selene recommend manager review or a formal notice path.

## 8. Performance-To-Reward Handoff

Selene may recommend:

- bonus eligibility,
- recognition,
- promotion review,
- pay review,
- training advancement.

Reward evidence may include:

- strong on-time completion,
- low rework,
- strong reliability,
- strong communication,
- high task rescue contribution,
- attendance reliability where lawful,
- consistent improvement trend,
- efficient shift performance.

Compensation/Payroll execute rewards only through protected approval.

PH1.HWM.PERFORMANCE provides evidence. It does not pay bonuses, alter salaries, promote employees, or change employment terms.

## 9. Required Logical Packets

These are future logical packets. This document does not claim they currently exist in repo truth.

### WorkloadPacket

- `person_id`
- `active_task_count`
- `due_this_week`
- `rostered_hours`
- `available_hours`
- `estimated_effort_load`
- `overload_score`
- `skill_match_score`
- `recommendation`

### SkillCapacityPacket

- `person_id`
- `skill_tags`
- `skill_level`
- `active_tasks`
- `available_hours`
- `rostered_hours`
- `overtime_risk`
- `workload_score`
- `performance_score`
- `recommended_assignment_fit`

### ReminderPlanPacket

- `reminder_plan_id`
- `task_id`
- `person_id`
- `reminder_type`
- `reminder_schedule`
- `risk_triggered`
- `next_reminder_at`
- `escalation_after`
- `cancelled_at`
- `completion_linked`

### ProgressRiskPacket

- `task_id`
- `risk_type`
- `risk_reason`
- `detected_at`
- `recommended_options`
- `originator_update_required`

### PerformancePacket

- `person_id`
- `period_start`
- `period_end`
- `tasks_assigned`
- `tasks_accepted`
- `tasks_completed`
- `tasks_completed_on_time`
- `tasks_completed_late`
- `tasks_failed`
- `tasks_ignored`
- `average_delay`
- `quality_score`
- `rework_count`
- `blocker_reporting_score`
- `communication_score`
- `reliability_score`
- `improvement_trend`
- `disciplinary_risk_level`
- `reward_eligibility_signal`

### PerformanceNoticePacket

- `person_id`
- `issue_type`
- `evidence_summary`
- `failed_task_refs`
- `support_actions_attempted`
- `manager_review_required`
- `notice_level`
- `improvement_period_start`
- `improvement_period_end`
- `next_review_at`
- `company_policy_ref`
- `authorized_by`
- `audit_ref`

### RewardEligibilityPacket

- `person_id`
- `period_start`
- `period_end`
- `evidence_summary`
- `on_time_completion_rate`
- `on_budget_completion_rate`
- `quality_score`
- `rework_rate`
- `attendance_reliability`
- `recommendation_type`
- `compensation_owner`
- `approval_required`
- `approval_status`
- `audit_ref`

## 10. Required Simulation Inventory

Required future simulations include:

- `WORKLOAD.CHECK_CAPACITY`
- `WORKLOAD.RANK_ASSIGNEES`
- `WORKLOAD.DETECT_OVERLOAD`
- `WORKLOAD.SUGGEST_REBALANCE`
- `PERFORMANCE.CALCULATE_SCORECARD`
- `PERFORMANCE.DETECT_REPEATED_DELAY`
- `PERFORMANCE.DETECT_REPEATED_FAILURE`
- `PERFORMANCE.PREPARE_MANAGER_REVIEW`
- `PERFORMANCE.RECOMMEND_SUPPORT`
- `PERFORMANCE.RECOMMEND_FIRST_NOTICE`
- `PERFORMANCE.TRACK_IMPROVEMENT_PERIOD`
- `PERFORMANCE.CLOSE_IMPROVEMENT_CASE`
- `PERFORMANCE.DETECT_REWARD_ELIGIBILITY`
- `PERFORMANCE.PREPARE_BONUS_RECOMMENDATION`

Simulation names are logical design targets pending Grand Architecture Reconciliation and repo-truth activation.

## 11. Examples

### Sarah Overloaded, Michael Better Fit

"Sarah has the required CAD skill, but she is overloaded this week. Michael has CAD experience, 12 available hours, and can complete the drawings by Friday without overtime. I recommend assigning it to Michael."

### David Repeated Failure

"David has missed 6 of 9 accepted tasks this month. Three had no valid blocker reason. Support was offered twice. I recommend manager review and first performance notice under company procedure."

Selene must not terminate David automatically.

### Worker Improves

If a worker improves during an improvement period, Selene should close the case, record the improved evidence, and stop escalation.

### Reward Eligibility

"Priya completed 96 percent of assigned tasks on time this quarter, had low rework, and repeatedly helped rescue blocked work. I recommend recognition or bonus review according to company policy."

Compensation/Payroll must execute any approved reward.

### Contractor Overrun Risk

"Michael has used 90 percent of approved contractor hours. Should I request approval for more hours, reduce scope, or stop work?"

Contractor commercial changes are protected.

## 12. What Must Not Happen

- no performance discipline without evidence,
- no punishment for impossible tasks,
- no hidden performance scoring used for HR action,
- no reward execution by Performance engine,
- no workload engine assigning tasks directly,
- no awareness engine sending messages directly,
- no negotiation engine mutating task/roster truth directly,
- no implementation from this document alone.
