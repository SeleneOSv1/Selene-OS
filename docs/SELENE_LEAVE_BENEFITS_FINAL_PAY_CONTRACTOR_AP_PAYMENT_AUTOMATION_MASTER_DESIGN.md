# Selene Leave, Benefits, Final Pay, Contractor/AP + Payment Automation Master Design

MASTER_DESIGN
NOT_RUNTIME_IMPLEMENTATION
PENDING_GRAND_ARCHITECTURE_RECONCILIATION

## 0. Authority and Scope

AGENTS.md controls execution.

This is a docs-only master design. No runtime code was changed. This document does not authorize Leave, Benefits, Final Pay, Contractor/AP, payment, bank, accounting, payroll, HR, roster, access, provider, migration, packet, client, adapter, or old-path implementation.

The Payroll/HR repo-truth extraction remains the factual base. Current repo truth does not prove complete Leave, Benefits, Final Pay, Contractor/AP, or Payment runtime owners. This document defines future architecture pending Grand Architecture Reconciliation.

## 1. Purpose

This document governs future:

- leave accumulation,
- leave payout,
- sick leave handling,
- holiday pay,
- final pay,
- employee advances/deductions,
- contractor payment,
- contract milestones,
- direct payment/accounting handoff.

It keeps employees, contractors, Payroll, HR, Finance/Accounting, AP, Access, Task/HWM, Roster/Attendance, and payment providers in separate owner lanes.

## 2. Company Leave Policy Setup

Companies differ. Selene must ask setup questions:

- Do annual leave days accumulate?
- Do unused annual leave days pay out?
- Do sick leave days accumulate?
- Do unused sick leave days pay out?
- Can leave be carried forward?
- Is leave paid at termination?
- Are public holidays paid?
- Are casual employees treated differently?
- Are part-time employees treated differently?
- Are full-time employees treated differently?
- Are contractors excluded?
- Can employees cash out leave?
- Are leave balances capped?
- What evidence is required for sick leave?
- How does leave interact with public holidays?

Selene stores approved company policy and uses it automatically through Leave/Benefits owners.

Example:

"Your company allows annual leave to accumulate and pays unused annual leave on resignation. Sick leave does not pay out. Is that correct?"

## 3. Leave Evidence Flow

Employee says:

"Selene, I'm sick today."

Selene:

1. records sick-day request,
2. checks roster,
3. checks policy,
4. asks evidence if required,
5. finds cover if needed,
6. updates payroll leave treatment through correct owners,
7. notifies manager if required,
8. audits.

Leave evidence must capture:

- absence type,
- requested dates/times,
- paid/unpaid classification,
- required proof,
- approval status,
- roster impact,
- attendance impact,
- payroll treatment,
- audit refs.

## 4. Public Holiday Handling

Public holiday handling must separate:

- holiday calendar owner,
- roster impact,
- attendance evidence,
- leave interaction,
- payroll pay treatment,
- finance cost impact.

Holiday cases:

- employee worked holiday,
- employee did not work holiday,
- holiday during leave,
- holiday on weekend,
- observed/substitute holiday,
- company override,
- regional mismatch.

Holiday pay cannot be calculated from an unconfirmed or stale calendar.

## 5. Final Pay Automation

When employee resigns or is terminated:

HR confirms end date.
Attendance confirms final worked hours.
Roster removes future shifts.
Task/HWM completes handover.
Leave calculates unused leave.
Compensation identifies owed bonus/commission if applicable.
Payroll calculates final pay.
Finance/Accounting processes payment.
Access removes company access.
Tax/reporting handoff is prepared.

Final pay may include:

- base pay to final date,
- approved overtime,
- unused annual leave,
- sick leave payout if company policy allows,
- bonus/commission owed,
- deductions,
- salary advance recovery,
- notice period treatment,
- severance if applicable,
- tax/contribution adjustment.

## 6. Handover Before Final Pay Guardrail

Before final payment, Selene may require policy-authorized checks:

- open tasks summarized,
- files handed over,
- assets returned,
- contractor/customer follow-ups transferred,
- final timesheet submitted,
- manager verifies handover.

Guardrail:

This is policy and jurisdiction dependent. Selene must not unlawfully delay statutory final pay. If legal final-pay deadline conflicts with handover requirement, Selene must escalate and route the lawful payment path while preserving handover evidence and manager follow-up.

Selene may guide:

"Before final pay can be completed, I need your open-task handover and final timesheet. I'll walk you through both."

Only lawful policy can decide whether that blocks final pay.

## 7. Employee Advances And Deductions

If employee received early payment:

- advance amount is recorded,
- deduction schedule is agreed,
- deduction appears in payroll draft,
- employee can request delay/change,
- approval may be required,
- minimum net pay protection may apply,
- missed repayment creates exception.

Employee says:

"Can you deduct that advance next month instead?"

Selene:

- checks policy,
- checks payroll status,
- checks approval requirements,
- checks tax/accounting treatment,
- explains effect,
- routes approval if needed.

Employee advance deductions must never be hidden.

## 8. Contractor Types

Selene must support:

- hourly contractor,
- lump-sum contractor,
- milestone contractor,
- retainer contractor,
- project contractor,
- site contractor,
- agency contractor,
- consultant.

Contractors are not employees by default.

## 9. Contractor Tracking

For each contractor, Selene tracks:

- contract start/end,
- approved hours,
- hours used,
- hourly rate,
- milestones,
- deliverables,
- task submissions,
- work progress,
- site attendance where lawful,
- invoice status,
- payment approval,
- access expiry.

Contractor overrun detection:

- warn at thresholds such as 80% or 90% of approved hours,
- route extra-hours approval,
- update Finance/AP exposure,
- notify internal manager.

## 10. Contractor Payment Flow

Hourly contractor:

contractor logs hours
-> Selene checks approved hours
-> Selene checks task/project evidence
-> Selene warns at 80% or 90%
-> manager approves overrun if needed
-> Finance/AP processes invoice/payment

Milestone contractor:

contractor submits milestone
-> Task/HWM records evidence
-> manager/verifier approves milestone
-> Finance/AP releases payment

Lump-sum contractor:

contract completion verified
-> contract owner approves
-> Finance/AP pays

Payroll does not pay contractors unless company policy explicitly routes contractor through payroll and repo-truth owner proof exists.

## 11. Contractor Access Expiry

When contract ends:

- Access expires permissions,
- Task/HWM checks deliverables,
- Finance/AP checks invoices,
- PH1.REM reminds manager before expiry,
- PH1.BCAST/DELIVERY sends required notices,
- audit records access expiry.

Selene must not leave contractor access active because humans forgot.

## 12. Direct Bank / Accounting Payment Integration

Future payment flow:

Payroll/AP prepares approved payment instruction
-> Accounting validates ledger
-> Finance approves if threshold requires
-> Bank/Payment Provider executes
-> confirmation returns
-> Selene notifies payee
-> audit and reconciliation complete

Payment may include:

- employee salary,
- final pay,
- contractor invoice,
- bonus,
- commission,
- expense reimbursement,
- salary advance,
- deduction recovery,
- super/pension/CPF remittance,
- tax remittance.

Accounts Payable / Accounting should own payment processing. Payroll owns employee pay instruction. AP owns supplier/contractor payment instruction. Finance owns budget/money approval. Bank provider executes transfer.

## 13. Tax And Reporting Handoff

Depending on country/region, Selene may prepare:

- tax reporting,
- superannuation/pension/CPF remittance,
- employer contribution reporting,
- termination reporting,
- year-end employee summaries,
- contractor tax reporting.

Actual reporting must be source-backed, owner-approved, and integrated through accounting/compliance systems.

## 14. What Must Not Happen

- no leave payout without company policy,
- no sick leave payout if policy forbids,
- no final pay blocked by handover where law requires payment,
- no contractor paid as employee by default,
- no contractor invoice paid without approval,
- no contractor overrun ignored,
- no contractor access left active,
- no direct bank transfer without approved payment rail,
- no tax reporting invented by GPT-5.5,
- no employee advance deduction hidden from employee,
- no implementation from this document alone.

## 15. Final Architecture Sentence

Selene Leave, Benefits, Final Pay, Contractor/AP, and Payment Automation ensures every absence, benefit, final payout, contractor hour, milestone, invoice, salary advance, deduction, payment instruction, tax/reporting handoff, and access expiry is evidence-driven, policy-governed, employee-visible where appropriate, and processed through the correct payroll, accounting, finance, access, and banking owners.
