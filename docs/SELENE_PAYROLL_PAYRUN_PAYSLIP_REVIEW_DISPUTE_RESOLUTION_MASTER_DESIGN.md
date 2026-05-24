# Selene Payroll Payrun, Payslip Review + Employee Dispute Resolution Master Design

MASTER_DESIGN
NOT_RUNTIME_IMPLEMENTATION
PENDING_GRAND_ARCHITECTURE_RECONCILIATION

## 0. Authority and Scope

AGENTS.md controls execution.

This is a docs-only master design. No runtime code was changed. This document does not authorize payroll implementation, payrun execution, payslip generation, payment provider integration, dispute runtime, migrations, packets, client edits, adapter edits, or old-path deletion.

The Payroll/HR repo-truth extraction remains the factual base. Current repo truth does not prove a complete Payroll runtime engine, payrun engine, payslip engine, or dispute engine. This document defines future payroll automation pending Grand Architecture Reconciliation.

## 1. Purpose

Selene must prepare accurate payroll automatically and interact directly with employees when issues arise.

Target sequence:

collect evidence continuously
-> prepare payroll draft
-> generate employee payslip preview where policy allows
-> send payslip to employee for review where policy allows
-> employee agrees or disputes
-> Selene resolves dispute using evidence
-> unresolved dispute escalates
-> approved payroll proceeds to payment
-> payment confirmation is sent

## 2. Payroll Evidence Inputs

For each employee, Selene needs:

- active employment status,
- bank account,
- tax/super/retirement/benefit details,
- base salary or hourly rate,
- working days,
- roster schedule,
- actual clock-in/out,
- approved overtime,
- leave/sick/unpaid days,
- public holiday treatment,
- commission,
- bonus,
- allowances,
- deductions,
- salary advance repayments,
- contract terms,
- final pay status if leaving.

All inputs must arrive through evidence packets from canonical owners. Payroll must not pull untrusted chat text into pay calculation.

## 3. Payrun Lifecycle

Required future lifecycle:

1. evidence collection,
2. missing/contradictory data detection,
3. direct employee issue resolution,
4. payroll draft,
5. exception review,
6. employee payslip preview where policy allows,
7. employee agrees or disputes,
8. dispute resolution,
9. approval,
10. payment instruction,
11. bank/accounting confirmation,
12. final payslip,
13. audit.

Payroll should be exception-first:

- complete verified evidence creates draft pay lines;
- missing/disputed/stale/contradictory evidence creates exceptions;
- protected approvals are routed only through authority and simulation.

## 4. Payslip Preview Before Payment

Before payment, Selene sends employees a payslip preview when policy allows.

Example:

"Your draft payslip for this period is ready. Net pay is $1,842.50. Please review it. If something looks wrong, tell me what you believe is incorrect."

Employee responses:

- agree,
- dispute,
- ask explanation,
- request correction,
- submit missing evidence.

If employee agrees, the payslip is marked employee-reviewed and payroll proceeds when company approval/payment rules pass.

If employee does not respond, company policy decides whether payroll proceeds after a review window. Silence must not be assumed as acceptance unless policy says so and the employee was given clear notice.

## 5. Pay Dispute Flow

Employee says:

"My pay is wrong."

Selene asks or infers:

- which pay period,
- which amount is disputed,
- whether the issue is hours,
- overtime,
- leave,
- commission,
- bonus,
- deduction,
- tax/super,
- bank/payment,
- final pay.

Selene checks:

- roster,
- clock-in/out,
- location evidence,
- approved overtime,
- leave classification,
- timesheet,
- task/milestone completion,
- commission/bonus rule,
- deductions,
- salary advance recovery,
- tax/contribution settings,
- bank account,
- payment status.

Dispute statuses are defined in the Evidence Fabric layer. Disputes must not become ad hoc chat.

## 6. Selene Resolves Directly Where Possible

Employee:

"I worked overtime Thursday but it's not included."

Selene checks attendance and approval evidence.

If attendance proves overtime worked but approval is missing:

"You did work the extra time, but it is pending approval. I'll send the approval request to your supervisor now."

If the system draft was wrong:

"You're right. Your approved overtime was not included in the draft. I've corrected the payroll draft and updated your payslip."

If employee is mistaken:

"I checked the roster, attendance, and approval record. Thursday's extra time was not approved as overtime, so it is not payable under the current company rule. I can escalate this if you want a manager review."

If unresolved, Selene creates a dispute case, attaches evidence, summarizes the employee claim, summarizes the system calculation, routes to Payroll/HR/Manager, and tracks resolution.

## 7. Payroll Correction Flow

Corrections must support:

- correction before payment,
- correction after payment,
- employee-favor correction,
- employer-favor correction,
- tax/contribution recalculation,
- payslip regeneration,
- payment adjustment,
- audit and employee-visible explanation.

Payroll correction must never be silent. Employee-visible and internal summaries must be separate where privacy requires.

## 8. Salary Advances And Deduction Reschedule Logic

If employee received an advance:

1. advance amount is recorded,
2. deduction schedule is created,
3. employee can ask for repayment change,
4. policy/approval checks apply,
5. deduction is applied to payroll.

Employee says:

"Can you deduct my advance next month instead?"

Selene checks company policy, remaining balance, manager/payroll approval requirement, minimum net pay protection, current payrun status, and cutoff. Then Selene explains whether the request can be prepared, approved, delayed, or refused.

Salary advances/deductions can be rescheduled only with policy and approval.

## 9. Accounting / Bank Integration

Future direct bank connection:

Selene prepares payment instruction
-> Accounting validates ledger impact
-> Finance approves if required
-> Bank/Payment Provider executes transfer
-> payment confirmation returns
-> employee is notified

Direct payment must be protected:

- authority,
- simulation,
- approval matrix,
- bank/payment provider proof,
- audit,
- reconciliation,
- failure handling.

## 10. Payslip Secure Delivery

Payslip delivery must define:

- delivery channel,
- identity check,
- download expiry,
- employee portal route,
- email/SMS restrictions,
- audit,
- wrong-recipient prevention,
- access-controlled retrieval,
- employee-visible confirmation.

PH1.BCAST/DELIVERY owns delivery. Payroll owns payslip truth. PH1.WRITE owns employee-safe explanation.

## 11. Payroll Troubleshooting Assistant

Selene must answer:

- why was my pay lower,
- where is my overtime,
- why was leave deducted,
- why was tax higher,
- which bank account was paid,
- when will payment arrive,
- why was my commission missing,
- why was my final pay different.

PH1.D/GPT-5.5 may help explain. Payroll evidence decides truth. PH1.WRITE produces final wording.

## 12. What Must Not Happen

- no payment without approval/payment authority,
- no payslip sent insecurely,
- no employee dispute ignored,
- no payroll correction without audit,
- no bank transfer without accounting/payment proof,
- no salary advance deduction hidden from employee,
- no employee-visible explanation exposing other employees' data,
- no GPT-5.5 final payroll calculation,
- no implementation from this document alone.

## 13. Final Architecture Sentence

Selene Payroll prepares payruns automatically from verified evidence, sends payslip previews for employee review where policy allows, resolves disputes through evidence-based troubleshooting, escalates unresolved issues, and sends approved payment instructions to Accounting/Banking with full audit.
