# Finance / Accounting Document 14 — Selene Period Close + Financial Reporting Engine

```text
DOCUMENT TYPE:
FINANCE / ACCOUNTING MASTER DESIGN

DOCUMENT NUMBER:
14

ENGINE:
PH1.PERIOD_CLOSE / PH1.FIN_REPORTING

FULL NAME:
Selene Period Close, Financial Reporting, Management Accounts, Board Pack, Compliance Pack, and Continuous Close Engine

STATUS:
FUTURE_CANONICAL_ARCHITECTURE
NOT_RUNTIME_IMPLEMENTATION
PENDING_REPO_TRUTH_ACTIVATION
PENDING_SIMULATION_MAPPING
```

## 0. Authority And Scope

AGENTS.md controls this document.

This is a docs-only architecture addition.

No runtime code is implemented by this document.

No schemas, migrations, APIs, packet structs, tests, or engine code are created by this document.

This document defines future canonical architecture for PH1.PERIOD_CLOSE / PH1.FIN_REPORTING. Repo-truth activation, simulation mapping, owner mapping, tests, and approved implementation slices must happen later before runtime behavior can be claimed.

## 1. Purpose

Selene Period Close + Financial Reporting Engine owns the process of turning all financial activity into clean, reviewable, auditable reports.

It answers:

```text
Is the accounting period ready to close?
Are bank reconciliations complete?
Are AP and AR reconciled?
Are payroll, tax, inventory, assets, depreciation, accruals, prepayments, and journals complete?
Are there unresolved exceptions?
Can we produce the P&L, balance sheet, cashflow report, budget variance report, and management accounts?
What still blocks close?
Who needs to fix it?
What can Selene close automatically?
What requires human review?
What reports should be sent to management, board, owners, investors, or regulators?
```

This is not a generate-report button.

This is the accounting control room.

A normal system says:

```text
Here is a P&L.
```

Selene says:

```text
The P&L is ready. Bank, AR, AP, payroll, tax, inventory, and accrual checks are complete. Two immaterial exceptions were auto-resolved under policy. One supplier credit note remains open but is disclosed in the close pack.
```

That is financial reporting with source readiness.

## 2. Core Selene Law

```text
Selene must run a continuous close, not a monthly panic ritual.

Routine checks and reconciliations should run automatically throughout the period.
Humans should only review exceptions, judgment areas, material adjustments, and final approval.
Reports must never pretend to be final if source engines are incomplete.
```

The goal:

```text
month-end should be confirmation, not archaeology.
```

Selene should already know what is ready, what is blocked, and who owns the blocker.

## 3. Engine Boundary

### 3.1 PH1.PERIOD_CLOSE / PH1.FIN_REPORTING Owns

```text
period close checklist
continuous close monitoring
close readiness scoring
financial reporting pack generation
management accounts
board reporting pack
owner/investor reporting pack
compliance reporting pack preparation
P&L reporting
balance sheet reporting
cashflow reporting
budget vs actual reporting
working capital reporting
aged debtors/creditors reporting
close exception worklist
close certification workflow
report version control
report distribution readiness
close audit evidence pack
```

### 3.2 PH1.PERIOD_CLOSE / PH1.FIN_REPORTING Does Not Own

```text
bank transaction proof
AP invoice validation
AR invoice creation
payroll calculation
inventory physical truth
asset lifecycle
depreciation policy ownership
tax law truth
payment execution
budget approval
board voting
ledger posting source truth
```

### 3.3 Correct Owner Split

```text
Accounting / GL = ledger, journals, accounting records.
BankRec = bank reconciliation proof.
AP = supplier invoice/payable truth.
AR = customer invoice/receipt truth.
Payroll = payroll calculation and payroll liabilities.
Tax = tax rules, tax returns, tax obligations.
Inventory = stock truth and inventory handoff.
Asset = asset lifecycle and depreciation evidence.
Budget = budget and variance.
Cashflow = liquidity forecasts.
Period Close = readiness, reporting, close pack, management accounts, board pack.
Access/Authority = report access and close approval.
Audit = proof trail.
PH1.WRITE/GPT-5.5 = human explanation and narrative drafting.
```

Period Close does not create truth.

It gathers truth, checks it, packages it, versions it, and tells humans whether it is safe to rely on.

## 4. Continuous Close Model

Selene should not wait for the end of the month.

She should perform continuous close checks daily or event-driven.

### 4.1 Continuous Checks

```text
bank feed synced
bank reconciliations current
supplier invoices processed
customer receipts matched
payroll runs posted/ready
tax obligations calculated or pending
inventory receipts/adjustments reviewed
asset additions/disposals reviewed
depreciation ready
accruals identified
prepayments identified
intercompany items identified if applicable
suspense accounts monitored
unmatched transactions reviewed
budget variances monitored
```

### 4.2 Close Readiness Score

Selene gives a readiness score:

```text
GREEN — ready
YELLOW — minor exceptions
ORANGE — material exceptions
RED — close blocked
BLACK — source data unreliable
```

Selene says:

```text
May close readiness is 87%. Bank and payroll are complete. AP has two supplier credit notes pending. Inventory has one stock variance above threshold.
```

## 5. Period Close Lifecycle

```text
PeriodOpen
ContinuousCloseMonitoring
PreCloseReview
CloseChecklistGenerated
SourceEngineValidation
ExceptionsDetected
AdjustmentsProposed
PendingReview
ReadyToClose
Closed
Locked
Reported
Reopened
Archived
```

### 5.1 PeriodOpen

Transactions flow normally.

### 5.2 ContinuousCloseMonitoring

Selene continuously checks readiness.

### 5.3 PreCloseReview

Selene warns before close date:

```text
The month closes in three days. I’m still waiting on two supplier statements and one bank feed sync.
```

### 5.4 CloseChecklistGenerated

Selene creates the close checklist by company type, size, industry, and active modules.

### 5.5 SourceEngineValidation

Selene asks each source engine for status.

### 5.6 ExceptionsDetected

Selene isolates material exceptions.

### 5.7 AdjustmentsProposed

Selene proposes accruals, prepaids, reclassifications, depreciation, FX, and routine close journals where source rules support it.

### 5.8 ReadyToClose

All blockers cleared or documented.

### 5.9 Closed

Accounting period is closed.

### 5.10 Locked

No changes except controlled reopening/adjustment.

## 6. Close Checklist

The close checklist depends on active modules.

Universal checklist:

```text
bank reconciliation complete
cash position verified
AP invoices processed
supplier credit notes reviewed
aged creditors reviewed
AR invoices processed
customer receipts matched
aged debtors reviewed
payroll processed
tax liabilities reviewed
suspense/unmatched items reviewed
accruals reviewed
prepayments reviewed
depreciation reviewed
management reports generated
audit pack generated
```

Conditional checklist:

```text
inventory valuation if stock exists
COGS review if products sold
WIP review if manufacturing active
recipe/food cost review if restaurant active
asset additions/disposals if assets exist
fleet costs if fleet active
insurance prepayments/claims if insurance active
intercompany if multi-entity active
foreign currency if multi-currency active
board pack if board reporting active
shareholder/investor pack if applicable
```

Selene should not ask small companies for irrelevant enterprise close tasks.

## 7. Source Engine Close Validation

Each source engine reports close status.

### 7.1 BankRec

```text
all bank accounts reconciled
unmatched transactions listed
cash position verified
payment provider settlements matched
```

### 7.2 AP / Creditors

```text
supplier invoices processed
credit notes applied/pending
supplier statements reconciled
aged creditors ready
AP holds listed
```

### 7.3 AR / Debtors

```text
invoices issued
receipts matched
credit notes/refunds processed
aged debtors ready
bad debt risk listed
```

### 7.4 Payroll

```text
payroll run completed
payroll liabilities recorded
employee payments confirmed
tax/super/pension obligations listed
```

### 7.5 Tax

```text
GST/VAT/sales tax draft position
payroll tax/withholding status
tax adjustments pending
filing due dates
```

### 7.6 Inventory

```text
stock movements processed
receipts accepted
stock variances reviewed
inventory valuation handoff ready
COGS handoff ready
damaged/expired/write-off items listed
```

### 7.7 Assets

```text
asset additions reviewed
disposals reviewed
depreciation ready
impairment flags listed
capital vs expense items reviewed
```

### 7.8 Budget

```text
budget vs actual ready
committed spend updated
material variances explained
```

## 8. Accruals And Prepayments

Selene must identify timing adjustments.

### 8.1 Accrual Candidates

```text
goods/services received but not invoiced
supplier work completed but invoice missing
utilities incurred but bill not received
payroll earned but unpaid
tax incurred
interest incurred
contract milestones completed
```

Selene says:

```text
The repair work was accepted before month-end but the supplier invoice has not arrived. I recommend an accrual.
```

### 8.2 Prepayment Candidates

```text
annual insurance paid upfront
software subscription paid yearly
rent paid in advance
maintenance contract prepaid
licences paid ahead
```

Selene says:

```text
This insurance payment covers 12 months. I recommend recognizing one month as expense and the rest as prepaid.
```

Accounting owns posting.

Selene proposes and routes.

## 9. Depreciation And Asset Close

Selene checks with Asset Engine and Accounting.

Close checks:

```text
new assets capitalized
asset receipts accepted
asset disposals recorded
depreciation run prepared
depreciation exceptions listed
impairment indicators flagged
asset under construction reviewed
vehicle/fleet assets linked
insurance/asset linkage reviewed
```

Selene says:

```text
Three assets were purchased this month. Two are ready for capitalization. One lacks receiving acceptance, so depreciation cannot start yet.
```

Depreciation should not start solely because an invoice exists.

## 10. Inventory And COGS Close

If inventory exists, close must include:

```text
stock movement completeness
receiving acceptance
inventory valuation
damaged stock
expired stock
write-offs
stock count variances
COGS calculation
WIP if manufacturing
raw materials if production/restaurant
```

Selene should flag:

```text
Inventory valuation is blocked because Warehouse B has an unresolved stock count variance.
```

Close does not pretend stock is correct while material stock variance remains unresolved.

## 11. Intercompany And Multi-Entity Close

If multi-entity exists, Selene supports:

```text
intercompany invoices
intercompany loans
intercompany expenses
management fees
transfer pricing entries
elimination candidates
multi-currency translations
entity-level close status
group close status
```

Selene says:

```text
Entity A is ready to close, but Entity B has unreconciled intercompany charges. Group reporting is not ready.
```

No group report should be finalized from half-closed entities.

## 12. Suspense And Unclassified Transactions

Selene must monitor suspense accounts and unclassified items.

Sources:

```text
unknown bank receipts
unknown payments
unclassified expenses
missing supplier/customer mapping
temporary clearing accounts
payment provider clearing
cash clearing
```

Selene action:

```text
auto-classify if high confidence and policy allows
propose classification
request evidence
escalate material unexplained items
block close if above threshold
```

Selene says:

```text
There are four suspense items totaling $180. Policy allows close with disclosure. One $8,000 unknown payment blocks close.
```

Not every small item deserves executive escalation. Material unexplained items do.

## 13. Financial Reports

Selene generates reports only when source status supports them.

Core reports:

```text
Profit and Loss
Balance Sheet
Cashflow Statement / Cash Movement Report
Trial Balance
General Ledger Summary
Budget vs Actual
Forecast vs Actual
Aged Debtors
Aged Creditors
Bank Reconciliation Summary
Inventory / COGS Summary
Payroll Summary
Tax Summary
```

Management reports:

```text
monthly management accounts
department performance
branch performance
product profitability
customer profitability
supplier cost trend
working capital report
cashflow forecast
variance explanations
KPI dashboard
```

Board reports:

```text
financial summary
cash position
budget variance
risk exceptions
major spend approvals
working capital
debt/loan status
capital expenditure
supplier/customer risk
forecast scenarios
management commentary
```

Selene must label reports:

```text
Draft
Preliminary
ClosePending
Final
BoardApproved
Filed
Superseded
```

Draft reports must be clearly labelled.

Final reports require source-engine readiness.

## 14. Report Version Control

Every report has:

```text
report_id
report_type
period
version
status
generated_at
source_data_cutoff
source_engine_status
adjustments_included
exceptions_disclosed
approved_by
distributed_to
audit_ref
```

If a report changes:

```text
new version
diff summary
reason
prior version retained
audit trail
```

Selene says:

```text
This is Version 3 of the May management accounts. It changed from Version 2 because the supplier credit note was received and AP balance reduced.
```

No mystery revisions.

## 15. Report Access And Distribution

Financial reports are sensitive.

Selene checks Access before showing or sending.

Access controls:

```text
who can view P&L
who can view balance sheet
who can view cash
who can view payroll summaries
who can view customer/supplier details
who can view board pack
who can export reports
who can distribute externally
```

Distribution channels:

```text
Selene portal
secure email
board portal
management dashboard
PDF export
Excel export
API/report pack
regulator filing pack where applicable
```

Selene should say:

```text
You do not have access to payroll detail. I can show the department-level payroll summary.
```

## 16. Management Commentary

Selene should generate human-readable commentary.

Inputs:

```text
P&L movement
budget variance
cashflow changes
working capital
supplier/customer issues
inventory changes
payroll changes
tax obligations
one-off events
```

GPT-5.5 may draft commentary, but source facts must be deterministic.

Example commentary:

```text
Revenue increased by 14% compared with last month, mainly due to B2B orders. Gross margin fell because supplier costs increased and discounts were used to clear slow-moving stock. Cash remains stable due to improved receivables collection.
```

That is board-readable.

## 17. Close Exception Worklist

Selene creates a worklist only for unresolved close blockers.

Exception fields:

```text
exception_id
source_engine
period
materiality
description
reason_code
owner
deadline
suggested action
status
audit_ref
```

Exception statuses:

```text
Open
Assigned
InProgress
Resolved
WaivedUnderPolicy
Escalated
Closed
```

Selene should not ask humans to inspect every clean section.

She should say:

```text
There are three close exceptions. Everything else is complete.
```

## 18. Close Approval

Close approval may require:

```text
accountant
finance manager
CFO
CEO
board
external accountant/adviser
```

Close approval states:

```text
NotReady
ReadyForPreparerReview
ReadyForFinanceReview
ReadyForCFOApproval
Approved
Locked
Reopened
```

Selene should support policy:

```text
small company → owner/accountant approval
mid-size → finance manager/CFO approval
enterprise → controller/CFO/board pack workflow
```

Routine close can be mostly automatic.

Final close approval remains authority-controlled.

## 19. Reopening A Closed Period

Closed periods should not be changed casually.

Reopen requires:

```text
reason
authority
affected reports
adjustment type
audit trail
version impact
distribution impact
tax/reporting impact
```

Selene says:

```text
Reopening May will supersede the management accounts already distributed. I need approval and a reason before proceeding.
```

No casual time travel in accounting.

## 20. Audit And Evidence Pack

Close pack includes:

```text
period status
bank reconciliation proof
AP reconciliation proof
AR reconciliation proof
payroll proof
tax proof
inventory valuation proof
asset/depreciation proof
journal summary
adjustments
exceptions
approvals
report versions
distribution record
audit hash/reference
```

Selene should be able to answer:

```text
Show me why May was closed.
```

And produce the evidence.

## 21. Autonomous Close Actions

Selene can automatically:

```text
monitor close readiness
create close checklist
request missing source statuses
auto-match routine close items
prepare accrual suggestions
prepare prepayment schedules
prepare depreciation run review
prepare variance explanations
generate draft reports
generate management commentary
prepare board pack
create exception worklist
send reminders to owners
close immaterial exceptions under policy
prepare final close approval request
archive close evidence
```

Selene requires authority for:

```text
period close approval
period reopen
material journal approval
material accrual/prepayment judgment
write-off
impairment
tax filing approval
board report release
external distribution
```

## 22. PH1.D / GPT-5.5 Role

GPT-5.5 should be heavily used for explanation, commentary, and report narration.

### GPT-5.5 May Help

```text
draft management commentary
summarize period results
explain variances
draft board pack narrative
summarize close exceptions
translate accounting language into plain English
draft report cover notes
prepare CFO briefing
```

### GPT-5.5 Must Not

```text
close period
approve reports
post journals
invent financial numbers
hide exceptions
override source engine status
approve tax filing
release board pack
change report version
```

GPT-5.5 writes like a human.

Selene deterministic engines decide if numbers are ready.

## 23. Human-Like Selene Interaction

### Pre-Close

```text
Month-end is in three days. Bank and payroll are ready. AP still has two supplier credit notes pending, and Inventory has one material stock variance.
```

### Close Readiness

```text
May is ready to close. All source engines are reconciled, and the remaining exceptions are immaterial and disclosed.
```

### Report Generation

```text
The management accounts are ready. Revenue increased, margin fell slightly, and cash improved due to faster collections.
```

### Close Blocked

```text
I cannot close the period yet. The operating bank account has an unmatched $8,000 outflow and AP has one unresolved supplier dispute.
```

### Reopen Warning

```text
Reopening April will supersede reports already sent to the board. Please provide a reason and approval.
```

Human-like. Calm. Direct.

## 24. State Machines

### Period Close State

```text
PeriodOpen
ContinuousCloseMonitoring
PreCloseReview
CloseChecklistGenerated
SourceEngineValidation
ExceptionsDetected
AdjustmentsProposed
PendingReview
ReadyToClose
Closed
Locked
Reported
Reopened
Archived
```

### Report State

```text
Draft
Preliminary
PendingSourceValidation
ReadyForReview
Final
Distributed
BoardApproved
Filed
Superseded
Archived
```

### Close Exception State

```text
Open
Assigned
InProgress
Resolved
WaivedUnderPolicy
Escalated
Closed
```

### Close Approval State

```text
NotReady
ReadyForPreparerReview
ReadyForFinanceReview
ReadyForCFOApproval
Approved
Rejected
Locked
Reopened
```

## 25. Reason Codes

```text
PERIOD_CLOSE_STARTED
CONTINUOUS_CLOSE_CHECK_COMPLETE
BANK_RECON_INCOMPLETE
AP_RECON_INCOMPLETE
AR_RECON_INCOMPLETE
PAYROLL_CLOSE_INCOMPLETE
TAX_CLOSE_INCOMPLETE
INVENTORY_VALUATION_INCOMPLETE
ASSET_DEPRECIATION_PENDING
SUSPENSE_ITEMS_OPEN
MATERIAL_EXCEPTION_OPEN
IMMATERIAL_EXCEPTION_WAIVED
ACCRUAL_RECOMMENDED
PREPAYMENT_RECOMMENDED
DEPRECIATION_READY
REPORT_DRAFT_CREATED
REPORT_PENDING_VALIDATION
REPORT_FINALIZED
PERIOD_READY_TO_CLOSE
PERIOD_CLOSED
PERIOD_LOCKED
PERIOD_REOPEN_REQUESTED
BOARD_PACK_READY
MANAGEMENT_ACCOUNTS_READY
```

## 26. Required Simulations

```text
continuous close readiness check
pre-close reminder
bank rec blocks close
AP credit note blocks close
AR receipt mismatch blocks close
payroll close complete
tax close pending
inventory variance blocks close
asset depreciation pending
accrual proposed
prepayment proposed
suspense item reviewed
management accounts generated
board pack generated
draft report labelled preliminary
final report generated
period close approved
period locked
period reopened with reason
report version superseded
audit close pack generated
```

## 27. Integration Map

```text
PH1.PERIOD_CLOSE / PH1.FIN_REPORTING
↔ PH1.ACCOUNTING / GL
↔ PH1.BANKREC / TREASURY
↔ PH1.CREDITORS / AP
↔ PH1.CREDITORS.RECON
↔ PH1.AR / DEBTORS
↔ PH1.AR.COLLECT
↔ PH1.PAYROLL
↔ PH1.TAX
↔ PH1.INVENTORY
↔ PH1.ASSET
↔ PH1.CASHFLOW
↔ PH1.BUDGET / PROFITABILITY
↔ PH1.BOARD
↔ PH1.SHAREHOLDER / INVESTOR ACCESS
↔ PH1.ACCESS / AUTHORITY
↔ PH1.AUDIT
↔ PH1.WRITE
↔ PH1.D / GPT-5.5
```

## 28. Required Logical Packets

```text
PeriodPacket
PeriodCloseChecklistPacket
SourceEngineCloseStatusPacket
CloseReadinessPacket
CloseExceptionPacket
AccrualProposalPacket
PrepaymentProposalPacket
DepreciationClosePacket
InventoryValuationClosePacket
FinancialReportPacket
ManagementAccountsPacket
BoardPackPacket
ReportVersionPacket
ReportDistributionPacket
CloseApprovalPacket
PeriodLockPacket
PeriodReopenPacket
CloseAuditPackPacket
```

Logical only. Codex maps later. Do not create packet structs from this document alone.

## 29. What Codex Must Not Do

```text
Do not let Period Close create source truth.
Do not let Period Close post journals directly.
Do not let GPT-5.5 invent financial commentary unsupported by source data.
Do not generate final reports if source engines are incomplete.
Do not hide unresolved exceptions.
Do not distribute sensitive reports without Access.
Do not reopen closed periods without authority.
Do not treat preliminary reports as final.
Do not implement from this document alone.
```

## 30. Final Architecture Sentence

Selene Period Close + Financial Reporting Engine is the continuous accounting close and reporting brain that monitors source-engine readiness throughout the period, validates bank, AP, AR, payroll, tax, inventory, assets, depreciation, accruals, prepayments, suspense items, budgets, and cashflow, creates close checklists and exception worklists, generates management accounts, financial statements, board packs, report versions, distribution controls, and audit evidence, while using GPT-5.5 for human-readable commentary and keeping source truth, ledger posting, authority, tax, and final approval under their proper deterministic engines.

Simple version:

```text
Selene watches the close all month.
Selene checks every source engine.
Selene finds blockers early.
Selene proposes accruals and prepayments.
Selene prepares reports.
Selene explains results like a human.
Selene shows what is final and what is draft.
Selene closes only when truth is ready.
Humans review exceptions and approve final close.
Everything is audited.
```

Month-end becomes a continuous, evidence-backed process where humans review judgment items instead of hunting missing invoices.
