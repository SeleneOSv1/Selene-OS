# Selene PH1.TASK + PH1.HWM.SCHEDULE — Task & Commitment Coordination Master Design

MASTER_DESIGN
NOT_RUNTIME_IMPLEMENTATION
PENDING_GRAND_ARCHITECTURE_RECONCILIATION

## 0. Authority and Scope

AGENTS.md controls execution.

This is a docs-only master design. No runtime code was changed. This document does not authorize implementation.

This document owns the Task/Schedule side of the PH1.HWM split: tasks, due dates, non-roster commitments, personal/household schedules, assignment, receiver acceptance, negotiation, progress, blockers, completion, verification, originator reporting, follow-up, and task rescue.

Current repo `PH1.SCHED` remains the WorkOrder retry/wait/fail scheduler. Human task schedule coordination is called `PH1.HWM.SCHEDULE` here pending Grand Architecture Reconciliation.

Future implementation requires repo-truth activation, approved file scope, tests, backend evidence, Access/Authority/Simulation proof where protected effects exist, PH1.WRITE validation proof, audit proof, and JD live acceptance where visible.

## 1. Executive Target

Task/Schedule Coordination manages:

- what needs to be done,
- who asked for it,
- who should do it,
- when it is due,
- whether it was accepted,
- whether it is progressing,
- whether it is blocked,
- whether it was completed,
- whether it was verified.

This is commitment management. It is not workforce roster publication, attendance, payroll, or staffing economics.

Selene must make a task a living operational commitment with accountable people, timing, status, rescue paths, follow-up, and outcome closure.

## 2. Owner Split

PH1.TASK owns:

- task truth,
- assignment,
- originator,
- receiver,
- verifier,
- deliverable,
- acceptance,
- progress,
- blockers,
- completion,
- verification,
- audit.

PH1.HWM.SCHEDULE owns:

- task due dates,
- task start windows,
- task check-in windows,
- recurring non-roster commitments,
- personal/household schedules,
- task deadline conflicts,
- task timeline planning.

PH1.REM owns reminder timing.

PH1.BCAST / PH1.DELIVERY owns delivery.

PH1.WORKLOAD owns capacity scoring.

PH1.ROSTER owns workforce shifts only.

PH1.WRITE owns final human-facing wording.

PH1.D / GPT-5.5 may propose meaning, clarification, and writing candidates. PH1.N may extract fields. PH1.X validates route, risk, and protected-action posture. Deterministic owners execute only after required gates pass.

## 3. Task Classes

PH1.TASK must support task classes so corporate work stays rigorous and personal coordination stays humane:

| Task Class | Purpose | Required Posture |
| --- | --- | --- |
| business critical task | high-value, compliance-sensitive, customer-impacting, HR-sensitive, payroll-adjacent, safety-related, or deadline-critical work | confirmation, acceptance, follow-up, blocker capture, verification where required, audit |
| normal business task | ordinary accountable work | assignment, acknowledgement or acceptance, progress, completion, audit |
| household task | chores, errands, school duties, pet care, shopping, family reminders | lightweight language, minimal corporate friction |
| personal reminder/task | self-owned task or reminder | simple scheduling and completion/dismissal |
| contractor deliverable task | task assigned to or tracked for a contractor | contractor scope, hours/overrun handoff where relevant |
| recurring task | repeated work or duty | recurrence, reminder, status, exception handling |
| blocked/dependency task | work waiting on person/system/material/authority | blocker reason, dependency tracking, originator update where required |

No task may bypass the lifecycle required for its class.

## 4. Task Lifecycle

Full business-critical lifecycle:

- Draft,
- Pending Originator Confirmation,
- Sent,
- Received,
- Pending Receiver Response,
- Accepted,
- Negotiation,
- Locked,
- In Progress,
- At Risk,
- Blocked,
- Completed Pending Verification,
- Verified Complete,
- Rework Required,
- Cancelled,
- Archived.

Normal business tasks may use a simpler accountable lifecycle:

- Draft or Created,
- Sent,
- Accepted or Acknowledged,
- In Progress,
- At Risk or Blocked where applicable,
- Completed Pending Verification where required,
- Complete,
- Archived.

Household tasks may use a lightweight lifecycle:

- Created,
- Sent or Reminded,
- Acknowledged or Not Acknowledged,
- Done or Not Done,
- Follow-Up if needed,
- Closed.

Personal reminders may use:

- Reminder Created,
- Reminder Delivered,
- Done, Dismissed, Snoozed, or Rescheduled,
- Closed.

A task is complete only when the task class's completion requirements pass. For work requiring verification, completion alone is not final until the authorized verifier approves the result.

## 5. Receiver Acceptance / Negotiation

A task is not accepted until the receiver confirms where the task class requires acceptance.

Receiver outcomes include:

- accepted on time,
- accepted with adjustment,
- declined,
- asks clarification,
- requests help,
- requests reassignment,
- marks blocked.

Selene manages negotiation but does not finalize protected changes without authority and simulation.

Negotiation may include:

- deadline change,
- scope reduction,
- helper assignment,
- reassignment,
- dependency resolution,
- schedule change proposal,
- receiver counterproposal,
- originator approval where required.

PH1.HWM.NEGOTIATION owns the structured negotiation process. PH1.TASK or PH1.HWM.SCHEDULE records approved outcomes depending on which truth surface changed.

## 6. Overload Protection

Selene must check workload before assignment where relevant.

If a receiver is overloaded, Selene must not blindly assign the task. It may recommend:

- renegotiate deadline,
- split task,
- add helper,
- reassign,
- reduce scope,
- escalate dependency.

Overload checks may consider:

- active task count,
- due dates,
- estimated effort,
- task complexity,
- rostered hours where relevant,
- actual hours where relevant,
- leave/sick status where available,
- meetings and travel,
- location/timezone,
- overtime risk,
- required skill,
- historical reliability.

PH1.WORKLOAD produces capacity and assignment-fit evidence. PH1.TASK does not become the workload engine.

## 7. Smart Allocation

Selene may suggest the best assignee based on:

- skill match,
- availability,
- current workload,
- rostered hours,
- leave,
- reliability,
- location,
- overtime risk,
- cost where relevant.

A smart allocation recommendation must explain the operational reason, not hide it.

Example:

Sarah has the required CAD skill, but she is overloaded this week. Michael has CAD experience, available capacity, and can finish before Friday without overtime. Selene may recommend Michael and ask for confirmation where required.

PH1.TASK records the approved task assignment. PH1.ROSTER remains the owner of workforce shifts. PH1.WORKLOAD remains the owner of capacity scoring.

## 8. Originator Reporting

The Originator must not be blind.

Immediate updates are required for:

- declined,
- blocked,
- at-risk,
- overdue,
- transfer requests,
- deadline change requests,
- completion requiring verification,
- failed verification,
- repeated non-response.

Originator reports should include:

- task title,
- receiver,
- current status,
- deadline,
- risk level,
- reason for delay where applicable,
- recommended next action,
- whether approval is required.

Routine updates may be batched or summarized for normal acceptance, low-risk progress, standard reminders, routine completion, and non-critical household status.

## 9. Failure Reason Capture

Selene must capture reason codes when a task is not completed, not store only vague free text.

Reason codes include:

- overloaded,
- unavailable,
- no acknowledgement,
- unclear scope,
- missing info,
- missing materials,
- dependency blocked,
- access missing,
- authority missing,
- unrealistic deadline,
- skill mismatch,
- no valid reason,
- management delay,
- supplier/customer delay,
- system issue.

Selene must not automatically blame the worker.

Failure classification must distinguish:

- person failure,
- planning failure,
- workload failure,
- management failure,
- dependency failure,
- business resource failure,
- skill mismatch,
- unrealistic deadline,
- unclear instruction,
- system access failure.

The solution depends on the cause.

## 10. Task Rescue Before Discipline

Selene first tries to rescue the task.

Rescue options include:

- clarify,
- checklist,
- missing info,
- helper,
- split,
- transfer,
- deadline extension,
- training suggestion.

Selene may also:

- ask the receiver what is blocking progress,
- request files or materials,
- ask the originator for a decision,
- assign a helper after approval where required,
- move lower-priority tasks away from the receiver,
- pair the receiver with a stronger worker,
- escalate a dependency.

Repeated failure may become performance evidence only after reason classification, support checks, workload reasonableness checks, and human review requirements are satisfied.

## 11. Household Mode

Household tasks must remain lightweight:

- chores,
- errands,
- school duties,
- pet care,
- shopping,
- family reminders.

Household mode should avoid corporate performance language such as disciplinary path, labour cost, staffing economics, formal verification, or performance notice unless the user specifically requests that structure.

Example:

"Alex has not confirmed the dog feeding reminder. I can remind him again at 6 PM or ask Sam to cover it."

## 12. Required Logical Packets

These are future logical packets. This document does not claim they currently exist in repo truth.

### TaskPacket

- `task_id`
- `domain`
- `task_class`
- `title`
- `description`
- `deliverable`
- `originator_id`
- `receiver_id`
- `verifier_id`
- `due_at`
- `priority`
- `estimated_effort`
- `cost_budget`
- `dependencies`
- `required_skills`
- `status`
- `risk_level`
- `audit_ref`

### TaskSchedulePacket

- `schedule_id`
- `task_id`
- `due_at`
- `planned_start_at`
- `planned_checkpoints`
- `calendar_conflicts`
- `recurring_rule`
- `schedule_risk`
- `reschedule_options`

### TaskFollowUpPacket

- `task_id`
- `receiver_id`
- `originator_id`
- `due_at`
- `reminder_plan_id`
- `next_check_at`
- `progress_required`
- `last_contact_at`
- `response_status`
- `risk_level`
- `escalation_path`
- `originator_update_required`

### TaskOutcomePacket

- `task_id`
- `assigned_to`
- `completed_at`
- `completed_on_time`
- `completed_on_budget`
- `completion_evidence`
- `verifier_id`
- `verification_status`
- `rework_required`
- `final_outcome`
- `failure_reason_code`
- `originator_notified_at`

### NegotiationPacket

- `task_id`
- `proposal_from`
- `proposed_change_type`
- `old_value`
- `new_value`
- `reason_code`
- `originator_approval_required`
- `approval_status`

### TaskTransferPacket

- `task_id`
- `current_receiver_id`
- `proposed_receiver_id`
- `transfer_reason_code`
- `current_progress_summary`
- `skill_match_score`
- `workload_comparison`
- `deadline_impact`
- `originator_approval_required`
- `approval_status`
- `audit_ref`

### OriginatorReportPacket

- `task_id`
- `originator_id`
- `report_type`
- `current_status`
- `risk_level`
- `blocker_reason`
- `recommended_options`
- `approval_required`
- `sent_at`
- `response_required_by`

## 13. Required Simulation Inventory

Required future simulations include:

- `TASK.CREATE_DRAFT`
- `TASK.CONFIRM_AND_SEND`
- `TASK.ACCEPT`
- `TASK.ACCEPT_WITH_ADJUSTMENT`
- `TASK.DECLINE`
- `TASK.NEGOTIATE_CHANGE`
- `TASK.LOCK_CONSTRAINTS`
- `TASK.MARK_BLOCKED`
- `TASK.REQUEST_HELP`
- `TASK.REASSIGN`
- `TASK.MARK_COMPLETED`
- `TASK.VERIFY_COMPLETION`
- `TASK.REJECT_COMPLETION`
- `TASK.CANCEL`
- `TASK.ARCHIVE`
- `SCHEDULE.CREATE_TASK_TIMELINE`
- `SCHEDULE.CHECK_CONFLICTS`
- `SCHEDULE.RESCHEDULE_TASK`
- `SCHEDULE.CREATE_RECURRING_COMMITMENT`
- `TASK.SCHEDULE_FOLLOW_UP`
- `TASK.SEND_PROGRESS_CHECK`
- `TASK.SEND_OVERDUE_REMINDER`
- `TASK.COLLECT_FAILURE_REASON`
- `TASK.NOTIFY_ORIGINATOR_STATUS`

Simulation names are logical design targets pending Grand Architecture Reconciliation and repo-truth activation.

## 14. Examples

### Drawings By Friday

User:

"Get someone to finish the drawings by Friday."

Selene should:

- understand the deliverable,
- detect required skill,
- check available staff,
- check current workload,
- check task schedule conflicts,
- check roster/leave where relevant,
- compare overtime risk where relevant,
- recommend best assignee,
- ask for confirmation where required,
- send task after approval,
- create follow-up plan,
- monitor progress,
- notify the originator where required,
- verify completion where required,
- close and log the outcome.

Expected response:

"Sarah can do it but is overloaded this week. Michael has CAD experience, 12 available hours, and can complete the drawings by Friday without overtime. Do you want me to assign it to Michael for Friday 4 PM?"

### Receiver Cannot Finish

Receiver:

"I can't finish this by Friday."

Selene should:

- ask or infer reason,
- check workload,
- find alternatives,
- prepare recommendation,
- ask the originator for approval where required.

Expected originator update:

"Michael cannot complete the task by Friday because he is assigned to two urgent production issues. Tom has the same CAD skill and available capacity. Do you want to transfer the task to Tom, add Tom as helper, or extend the deadline?"

### Household Task

User:

"Ask Alex to feed the dog tonight."

Selene may respond:

"I'll remind Alex tonight. If he does not confirm, I can remind him again or ask someone else to cover it."

### Task Versus Roster

"Move Michael's drawing deadline to Friday" = task schedule.

"Move Michael's Friday shift to Saturday" = roster.

Selene must not confuse the two.

## 15. What Must Not Happen

- no task assignment from LLM alone,
- no receiver overload ignored,
- no protected reassignment without authority,
- no discipline from task failure alone,
- no roster mutation from PH1.TASK,
- no reminder timing owned by PH1.TASK,
- no delivery owned by PH1.TASK,
- no implementation from this document alone.
