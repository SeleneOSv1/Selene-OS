# Selene Payroll + HR Automation Umbrella Master Design

MASTER_DESIGN
NOT_RUNTIME_IMPLEMENTATION
PENDING_GRAND_ARCHITECTURE_RECONCILIATION

## 0. Authority and Scope

AGENTS.md controls execution.

This is a docs-only master design. No runtime code was changed. This document does not authorize implementation, migrations, packets, schemas, providers, payroll execution, HR execution, payment execution, client edits, adapter edits, or old-path deletion.

The Payroll/HR repo-truth extraction remains the factual base: current repo truth does not prove a complete Payroll runtime engine, complete HR runtime engine, or Compensation business engine. Payroll is currently partial as protected intent, DRAFT simulation catalog shape, and architecture intent. HR is mostly absent as a runtime owner. Current `PH1.COMP` is deterministic computation, not compensation.

This document defines future Payroll + HR automation architecture pending Grand Architecture Reconciliation.

## 1. Purpose

Selene must not behave like a normal payroll system where humans manually collect timesheets, check rosters, chase staff, calculate overtime, fix bank accounts, prepare payslips, and then ask payroll to behave.

Selene's payroll/HR model is evidence-driven:

- Selene continuously collects work evidence.
- Selene continuously validates employee data.
- Selene tracks scheduled work and actual work.
- Selene tracks overtime, leave, holidays, commissions, bonuses, deductions, advances, benefits, and tax obligations.
- Selene prepares accurate payroll automatically from verified operational evidence.
- Selene sends payslips for review where policy allows.
- Selene resolves employee disputes directly where possible.
- Selene escalates unresolved or protected issues.
- Selene sends approved payment instructions to Accounting/Banking through correct owners.
- Selene audits everything.

The target is automation, not spreadsheet preparation.

## 2. Master Automation Law

Selene prepares payroll from operational truth.

Humans configure policy, approve protected exceptions, and resolve escalated disputes.

Selene does the daily chasing, checking, explaining, calculating, reminding, and evidence collection.

Protected payroll, HR, compensation, payment, access, banking, leave approval, final pay, contractor payment, salary change, and termination actions require deterministic owners, simulation, authority, confirmation where required, and audit.

## 3. Master Owner Split

Selene Orchestration coordinates the full flow. It must not own payroll truth, HR truth, roster truth, task truth, compensation truth, bank truth, accounting truth, contractor payment truth, or access truth.

| Owner | Owns | Must Not Own |
|---|---|---|
| HR | employee profile, employment status, start date, probation, resignation, termination, retirement, rehire, HR records, formal notices, employment lifecycle | pay calculation, bank payment execution, roster truth, access grants |
| Payroll | gross pay, net pay, payrun, payslip, payroll deductions, payroll tax/contribution calculations, final pay, corrections, employee pay instruction, payroll audit | employment lifecycle, task truth, roster truth, bank rails |
| Attendance / Timesheet | clock-in/out, breaks, actual hours, late start, early finish, approved overtime evidence, location evidence, timesheet evidence | pay calculation |
| Roster / Scheduler | working days, shifts, start/end time, work location, public holiday scheduling impact, roster group, schedule group | actual attendance and pay |
| Task / HWM | submitted work, completed tasks, contract milestones, progress reports, handover requirements, performance evidence | pay, employment status, roster truth |
| Compensation | base salary, hourly rate, overtime multiplier, weekend rate, public holiday rate, commission formula, bonus eligibility, allowances, benefits, approved salary changes, approved pay overrides | payrun execution or bank transfer |
| Finance / Accounting | payment processing, bank transfer rails, accounting ledger, tax reporting handoff, super/pension/CPF remittance, payroll journal, accounts payable, accounts receivable, salary advance recovery | employee payroll truth or HR truth |
| Access / Governance | salary visibility, bank/tax/health/super field access, payroll approval permissions, overtime approval permissions, termination authority, private field access | payroll calculation or HR lifecycle truth |
| PH1.D / GPT-5.5 | messy request understanding, explanations, drafts, dispute summaries, missing-data guidance | final payroll calculation, final legal/tax truth, protected execution |
| PH1.N | field and intent extraction candidates | authority or execution |
| PH1.X | route/risk/gate validation | domain truth mutation |
| PH1.WRITE | final human explanation, sensitive wording, denial/confirmation phrasing | payroll/HR state |
| PH1.REM | timing of reminders and follow-ups | payroll/HR source truth |
| PH1.BCAST / PH1.DELIVERY | notifications, delivery attempts, secure message handoff | payroll/HR source truth |

## 4. Evidence-Driven Payroll Flow

For every pay period, Selene gathers evidence from canonical owners:

- employee active status,
- salary or hourly rate,
- approved working schedule,
- actual attendance,
- approved overtime,
- leave and sick days,
- public holidays,
- task/milestone completion,
- commission and bonus events,
- salary advances,
- deductions,
- tax/super/retirement/benefit rules,
- bank account,
- payment method,
- accounting/payment readiness.

Selene then prepares payroll from the evidence bundle. Humans review exceptions only.

Example:

Tom was scheduled Monday-Friday, 7 AM-3 PM. Tom clocked in and out correctly. Tom worked two approved overtime hours. Tom completed assigned dispatch tasks. Tom had no leave. Tom has valid bank and tax details. Selene can prepare Tom's payroll line automatically, subject to active policy, rule, access, and audit gates.

## 5. Bank Account Setup And Change Summary

During onboarding, Selene may collect employee bank/payment details through PH1.ONB and Payroll-owned field policies:

1. Employee enters bank account.
2. Selene confirms account details.
3. Selene explains salary will be paid to that account.
4. Employee confirms.
5. Step-up verification occurs where required.
6. Bank details are stored under Payroll/Access controls.
7. Old records remain historical if changed later.

Employee bank-account change flow:

1. Employee says: "Selene, change my bank account."
2. Selene identifies employee.
3. Step-up verification is required.
4. Selene collects the new account.
5. Selene confirms old-vs-new with safe masking.
6. Payroll updates the payment profile only through approved simulation.
7. Old account remains in history.
8. Cutoff logic decides whether the current or next payrun uses the new account.
9. Audit records all steps.

Bank account change must never be completed from a casual chat instruction alone.

## 6. Employee Self-Service Summary

Employees should work directly with Selene where policy allows. They may ask to:

- change bank account,
- update phone number,
- change address,
- show payslip,
- ask why pay was lower,
- request overtime,
- request leave,
- report sick day,
- ask about tax/super/benefits,
- submit missing timesheet evidence,
- query final pay,
- ask where payment was sent,
- request deduction or salary advance adjustment.

Risk classes:

- low-risk profile updates may need identity confirmation and audit;
- sensitive payroll/HR changes require step-up verification;
- payroll-cutoff-sensitive changes require cutoff decision and employee explanation;
- manager/payroll/finance approval is required where policy says so.

New data becomes current. Old data becomes historical. Sensitive data is never silently erased.

## 7. Payment Execution Summary

Selene prepares payment instructions after payroll approval.

Flow:

payrun approved
-> payslips generated
-> employee review/dispute window where policy requires
-> exceptions resolved
-> payment file/payment instruction created
-> Accounting/Finance validates ledger
-> Bank/Payment Provider executes transfer
-> confirmation returned
-> employees notified
-> audit completed

Direct bank connection is future Accounting/Payment Provider integration. Selene may drive the process, but bank rails, accounting ledger, and payment execution must be governed by the correct owners.

## 8. Exception-First Payroll Principle

The payroll operator should not prepare ordinary payroll line by line when evidence is complete. Selene should surface exceptions:

- missing clock-in/out,
- unapproved overtime,
- leave without classification,
- bank account missing or stale,
- tax rule missing,
- pay rule mismatch,
- public holiday conflict,
- disputed payslip,
- late bank change,
- contractor invoice mismatch,
- final pay evidence missing,
- payment provider failure.

Routine complete evidence should flow to draft payroll. Exceptions require human review, owner validation, or protected approval.

## 9. What Must Not Happen

- no payroll based on manual guesses,
- no bank account change without step-up,
- no old employee data erased,
- no payrun with hidden exceptions,
- no payment transfer without payment-owner proof,
- no private salary/bank/tax fields exposed without Access,
- no GPT-5.5 payroll calculation as final truth,
- no employee termination without protected HR process,
- no contractor paid like employee by accident,
- no Selene Orchestration becoming a duplicate truth owner,
- no implementation from this document alone.

## 10. Final Architecture Sentence

Selene Payroll + HR Automation turns payroll into a continuous evidence-driven operating system: Selene gathers roster, attendance, tasks, leave, compensation, tax, benefit, banking, and accounting facts, resolves issues directly with employees where allowed, escalates only exceptions, and produces accurate protected payroll and payment handoffs with full audit while canonical engines retain their own truth.
