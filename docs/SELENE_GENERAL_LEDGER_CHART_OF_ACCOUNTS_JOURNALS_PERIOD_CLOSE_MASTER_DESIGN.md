# Selene General Ledger + Chart of Accounts + Journals + Period Close Master Design

```text
DOCUMENT TYPE:
MASTER DESIGN / GENERAL LEDGER + CHART OF ACCOUNTS + JOURNAL ENGINE + PERIOD CLOSE

STATUS:
MASTER_DESIGN
NOT_RUNTIME_IMPLEMENTATION
PENDING_GRAND_ARCHITECTURE_RECONCILIATION

PURPOSE:
Define Selene's accounting book engine: the General Ledger, Chart of Accounts, Journal Entry system, accounting periods, trial balance, period close, corrections, reversals, audit, and financial-statement foundation.
```

## 0. Authority and Scope

AGENTS.md controls execution.

This is a docs-only master design.

No runtime code was changed.

This document does not authorize implementation.

This document is Finance/Accounting Design Batch 1. It defines future General Ledger, Chart of Accounts, journal, posting, period close, correction, reversal, and accounting-book behavior. It must be read with the Finance + Accounting Autonomous Umbrella and the related chart-governance addendum.

Current repo truth does not prove a complete runtime General Ledger, Chart of Accounts, journal entry, period close, or financial statement engine. This is future architecture pending Grand Architecture Reconciliation.

## 1. Executive Target

The General Ledger is Selene's official accounting book.

Everything financial eventually lands here.

```text
Sales
purchases
payroll
contractors
bank payments
customer receipts
credit cards
assets
depreciation
inventory
tax
expenses
refunds
loans
leases
dividends
intercompany activity
```

The goal:

```text
Every valid financial event becomes a properly classified journal entry.

Every journal entry posts to the General Ledger.

Every ledger entry traces back to source evidence.

Every accounting period can be reviewed, closed, locked, reported, reopened only with authority, and audited.
```

Selene must not treat accounting like a dumping ground. The GL is not a junk drawer with account codes. It is the company's financial spine.

## 2. Master Law

```text
No ledger posting without source evidence.

No journal posting without a valid accounting period.

No account code without a valid Chart of Accounts entry.

No period close while required subledgers are unresolved.

No journal reversal without audit.

No manual journal without reason, authority, and evidence.

No GPT-5.5 final account posting.

No client, adapter, or UI may own ledger truth.
```

Draft journals are not financial statement truth.

Posted journals are not silently edited.

Corrections use reversal, adjustment, approved reclassification, or controlled period reopening.

## 3. Owner Split

### Accounting owns

```text
Chart of Accounts
General Ledger
journal entries
ledger postings
trial balance
period close
period lock
manual journals
automatic journals
reversing journals
corrections
financial statement base
accounting audit evidence
```

### Finance/Budget owns

```text
budget meaning
forecast meaning
profitability meaning
cashflow meaning
spend authority
board approval
```

Finance uses ledger data. Finance does not own the ledger.

### AP owns

```text
supplier bills
contractor invoices
installments
scheduled outgoing liabilities
```

AP creates accounting evidence. Accounting posts journal entries.

### AR owns

```text
customer invoices
receipts
credit notes
collections
debtor aging
```

AR creates accounting evidence. Accounting posts journal entries.

### Payroll owns

```text
employee pay calculation
payrun
payslip
employee pay instruction
payroll evidence
```

Payroll sends accounting handoff. Accounting posts payroll journals.

### Banking owns

```text
bank feeds
bank payments
payment confirmations
bank reconciliation evidence
```

Banking confirms money movement. Accounting posts and reconciles.

### Tax owns

```text
GST/VAT/sales tax treatment
tax codes
tax period evidence
tax reporting packs
```

Accounting consumes tax treatment. Accounting does not invent tax law.

### PH1.D / GPT-5.5 may assist

```text
suggest account classification
explain journal meaning
summarize close issues
draft user-facing explanations
help classify messy descriptions
```

But PH1.D must not post journals, close periods, override accounts, or invent accounting rules.

### PH1.WRITE owns final wording

```text
journal explanation
period close summary
variance explanation
close-blocker explanation
manual journal reason explanation
safe user-facing accounting messages
```

## 4. Chart of Accounts

The Chart of Accounts is Selene's financial language.

Every company must have a governed chart.

### Core account types

```text
Assets
Liabilities
Equity
Revenue
Cost of Goods Sold
Expenses
Other Income
Other Expenses
Contra Accounts
Clearing Accounts
Suspense Accounts
Tax Accounts
Payroll Liability Accounts
```

### Account fields

```text
account_id
tenant_id
legal_entity_id
country
account_code
account_name
account_type
parent_account_id
normal_balance: debit / credit
active_status
allow_direct_posting: true / false
system_controlled: true / false
tax_mapping_ref
reporting_mapping_ref
group_reporting_mapping_ref
cashflow_category_ref
effective_from
effective_to
created_by
approved_by
audit_ref
```

### Account hierarchy example

```text
1000 Assets
  1100 Cash and Bank
    1110 Operating Bank Account
    1120 Payroll Bank Account
    1130 Credit Card Clearing
  1200 Accounts Receivable
  1300 Inventory
  1400 Fixed Assets
    1410 Motor Vehicles
    1420 Equipment
    1430 Computers

2000 Liabilities
  2100 Accounts Payable
  2200 Payroll Liabilities
  2300 GST/VAT Payable
  2400 Loans and Lease Liabilities

3000 Equity
  3100 Share Capital
  3200 Retained Earnings
  3300 Dividends Declared

4000 Revenue
  4100 Sales Revenue

5000 Cost of Goods Sold

6000 Expenses
  6100 Wages Expense
  6200 Rent
  6300 Vehicle Expense
  6400 Contractor Expense
  6500 Depreciation Expense
```

### Account rules

```text
Some accounts are system-controlled.
Some accounts are direct-posting blocked.
Some accounts require subledger evidence.
Some accounts require tax code.
Some accounts require cost center.
Some accounts require project/customer/supplier ref.
Some accounts require finance approval.
Some accounts require accountant-only access.
```

Example:

```text
Accounts Receivable control account should not accept casual manual posting.

AR subledger creates the evidence.
Accounting posts the summary or linked journal.
```

This stops people from "just fixing AR" with a mystery journal.

## 5. General Ledger

The General Ledger stores posted accounting truth.

### Ledger posting fields

```text
ledger_posting_id
journal_entry_id
journal_line_id
tenant_id
legal_entity_id
account_id
account_code
debit_amount
credit_amount
currency
functional_currency_amount
reporting_currency_amount
fx_rate_ref
accounting_period_id
posting_date
source_engine
source_event_id
source_document_ref
counterparty_ref
cost_center_ref
department_ref
project_ref
location_ref
tax_code_ref
audit_ref
created_at
posted_at
posted_by
```

### Ledger law

```text
The General Ledger records posted truth.

Drafts do not affect financial statements.

Only posted journal entries affect ledger balances.

Posted entries may not be silently edited.

Corrections require reversal, adjustment, or controlled reopening.
```

## 6. Journal Entry Engine

A journal entry is the bridge between evidence and ledger.

### Journal entry types

```text
automatic_journal
manual_journal
system_generated_journal
payroll_journal
ap_journal
ar_journal
bank_journal
inventory_journal
asset_depreciation_journal
tax_journal
reversing_journal
adjustment_journal
closing_journal
intercompany_journal
consolidation_journal
```

### Journal header fields

```text
journal_entry_id
tenant_id
legal_entity_id
journal_type
source_engine
source_event_id
source_document_ref
description
accounting_period_id
journal_date
posting_date
status
created_by
approved_by
posted_by
reversal_of_journal_id
reversing_journal_id
requires_approval
approval_status
audit_ref
```

### Journal line fields

```text
journal_line_id
journal_entry_id
line_number
account_id
account_code
debit_amount
credit_amount
currency
tax_code_ref
counterparty_ref
cost_center_ref
department_ref
project_ref
location_ref
description
evidence_ref
```

### Journal states

```text
Draft
PendingEvidence
PendingApproval
Approved
Posted
Rejected
Reversed
Corrected
Locked
Archived
```

### Posting validation

Before posting:

```text
debits equal credits
valid accounting period
valid legal entity
valid account codes
account active on posting date
direct posting allowed or system source allowed
required tax code present
required cost center present
required source evidence present
required approval passed
period not closed/locked
currency conversion available if needed
audit ready
```

If any validation fails:

```text
JournalExceptionPacket:
  journal_entry_id
  reason_code
  blocking_field
  required_owner
  suggested_fix
  audit_ref
```

## 7. Automatic Journals

Selene should generate journals automatically from source events where evidence is strong.

### POS sale

```text
Source:
POS sale completed.

Journal:
Debit Bank/Card Clearing
Credit Sales Revenue
Credit GST/VAT Payable
Debit Cost of Goods Sold
Credit Inventory
```

### Supplier bill

```text
Source:
Supplier bill approved.

Journal:
Debit Expense / Inventory / Asset
Debit GST/VAT Receivable if claimable
Credit Accounts Payable
```

### Payroll

```text
Source:
Payroll committed.

Journal:
Debit Wage Expense
Debit Employer Contribution Expense
Credit Payroll Payable
Credit Tax Withholding Payable
Credit Super/Pension/CPF Payable
```

### Depreciation

```text
Source:
Monthly depreciation schedule.

Journal:
Debit Depreciation Expense
Credit Accumulated Depreciation
```

### Bank payment

```text
Source:
Bank confirms supplier payment.

Journal:
Debit Accounts Payable
Credit Bank
```

## 8. Manual Journals

Manual journals must be controlled.

Allowed use cases:

```text
accountant adjustment
period-end accrual
prepayment adjustment
correction
reclassification
FX adjustment
intercompany adjustment
tax adjustment
opening balance
auditor adjustment
```

Manual journal requirements:

```text
reason required
source evidence required
accountant/finance authority required
approval required where policy says
no posting to blocked control accounts unless elevated policy allows
audit required
PH1.WRITE explanation required if user-facing
```

Selene should ask:

```text
What is the reason for this journal, and what evidence supports it?
```

Selene must not casually respond:

```text
Sure, I'll post that.
```

That is how accounting systems become confession exhibits.

## 9. Accruals, Prepayments, and Reversing Journals

Selene must support period-end accounting.

### Accruals

```text
Expense incurred but supplier bill not received.

Debit Expense
Credit Accrued Expense Liability
```

### Prepayments

```text
Payment made before expense period.

Debit Prepaid Asset
Credit Bank/AP

Then monthly:
Debit Expense
Credit Prepaid Asset
```

### Reversing journals

Some journals reverse automatically next period.

```text
reversal_required: true
reversal_date
reversal_period_id
reversal_status
linked_reversing_journal_id
```

Example:

```text
Accrue electricity expense in May.
Reverse automatically on June 1.
Supplier bill arrives in June and posts normally.
```

## 10. Accounting Periods

Selene must manage accounting periods.

### Accounting period fields

```text
accounting_period_id
legal_entity_id
period_start
period_end
fiscal_year
period_number
status
opened_at
closed_at
locked_at
closed_by
reopened_by
audit_ref
```

### Period states

```text
NotOpened
Open
SoftClose
PendingClose
Closed
Locked
Reopened
Archived
```

### Period rules

```text
Open periods allow normal posting.

SoftClose allows limited posting with review.

PendingClose blocks normal automated postings but allows close adjustments.

Closed periods block normal posting.

Locked periods block all posting except controlled reopen/reversal path.

Reopening requires authority, reason, audit, and may require board/accountant approval.
```

## 11. Period Close

Period close is a formal process.

Selene should run close like an accounting supervisor who actually remembers every checklist item.

### Close checklist

```text
AP bills reviewed
AR invoices reviewed
bank reconciliation completed
credit card reconciliation completed
payroll journals posted
inventory movements posted
COGS posted
fixed asset depreciation posted
loan/lease interest posted
tax/GST/VAT entries reviewed
accruals posted
prepayments posted
intercompany balances reviewed
unmatched transactions resolved or explained
suspense account reviewed
manual journals approved
trial balance balanced
financial reports generated
period close approval obtained
```

### Close blocker examples

```text
unreconciled bank transaction
unapproved supplier bill
unmatched credit card transaction
unposted payroll journal
inventory adjustment pending
missing tax code
unapproved manual journal
open suspense balance
unresolved intercompany mismatch
```

### Selene close summary

```text
May 2026 is not ready to close.

Three blockers remain:
1. Two unreconciled bank transactions.
2. Payroll journal PAY-2026-05 is not posted.
3. One supplier bill is missing a tax code.

I can prepare the fixes or route them to the correct owners.
```

## 12. Trial Balance

Selene must produce trial balance from posted ledger.

### Trial balance fields

```text
trial_balance_id
legal_entity_id
accounting_period_id
account_id
account_code
opening_debit
opening_credit
period_debit
period_credit
closing_debit
closing_credit
currency
generated_at
source_ledger_hash
audit_ref
```

Validation:

```text
total_debits = total_credits
all accounts valid
period status known
no unposted journals included
source ledger trace exists
```

If trial balance does not balance, period cannot close.

## 13. Financial Statement Base

Document 2 does not fully design reporting, but GL must provide the base.

Financial statements built from ledger:

```text
Profit and Loss
Balance Sheet
Cash Flow Statement
Statement of Changes in Equity
Trial Balance
General Ledger Detail
Journal Report
Tax Summary Inputs
```

Reporting mappings must support:

```text
local statutory reporting
management reporting
group reporting
country reporting
multi-entity consolidation
```

Future Tax / GST / VAT / Country Compliance expands tax.

Future Multi-Entity + Multi-Currency + Consolidation expands consolidation.

Future Equity + Shareholder Distributions + Dividends expands dividends and equity distributions.

Those future documents are not written in this batch.

## 14. Corrections and Reversals

Posted journals must not be silently edited.

Correction options:

```text
reversal journal
adjustment journal
correcting journal
reopen period with authority
void draft before posting
reject pending journal
```

### Reversal flow

```text
user requests reversal
-> PH1.X classifies protected accounting action
-> Access checks authority
-> Accounting checks period status
-> Selene asks reason
-> reversal journal drafted
-> approval if required
-> reversal posted
-> audit linked to original journal
```

### Reversal packet

```text
JournalReversalRequestPacket:
  original_journal_id
  requester_id
  reason_code
  requested_reversal_date
  accounting_period_id
  period_status
  authority_required
  approval_status
  audit_ref
```

## 15. Suspense and Clearing Accounts

Selene needs suspense and clearing accounts, but they must be controlled.

Examples:

```text
unmatched bank receipt
unknown customer payment
unknown supplier payment
unclear expense category
payment provider clearing
card settlement clearing
payroll clearing
inventory clearing
```

Rules:

```text
suspense accounts are temporary
aging must be tracked
open suspense must block or warn at period close
high-value suspense requires escalation
old suspense requires review
```

Selene says:

```text
There is one $4,200 bank receipt sitting in suspense because I cannot match the customer. I need this resolved before period close.
```

## 16. Dimensions: Cost Centers, Projects, Locations, Departments

Journals should support dimensions.

```text
cost_center_id
department_id
project_id
location_id
customer_id
supplier_id
employee_id
asset_id
inventory_item_id
legal_entity_id
country
region
```

Dimensions allow Selene to answer:

```text
Which department is over budget?
Which project is profitable?
Which warehouse costs more?
Which customer has high service cost?
Which employee/card created this expense?
Which asset produced this depreciation?
```

Finance/Budget owns interpretation. Accounting stores the coded truth.

## 17. Multi-Currency Basics

Future Multi-Entity + Multi-Currency + Consolidation covers full multi-currency and consolidation, but GL must store currency correctly.

Each journal line should support:

```text
transaction_currency
transaction_amount
functional_currency
functional_amount
reporting_currency
reporting_amount
fx_rate_ref
fx_rate_date
fx_gain_loss_account_ref
```

Rules:

```text
No currency conversion without FX rate evidence.
No silent FX adjustment.
FX differences must be posted through approved rules.
```

## 18. Automation Logic

Selene should continuously watch accounting readiness.

She should automatically:

```text
draft journals from evidence
post journals when rules are certain and posting is allowed
flag missing account mappings
flag tax-code issues
flag unbalanced journals
flag closed-period posting attempts
prepare close checklist
route approval requests
prepare reversal drafts
summarize close blockers
generate trial balance
prepare reporting base
```

Selene should not automatically:

```text
post uncertain manual journals
close periods without approval
reopen locked periods
override tax codes
post to restricted control accounts
write off suspense balances
hide unmatched transactions
```

## 19. PH1.D / GPT-5.5 Role

Allowed:

```text
suggest account category from invoice description
explain journal entry in plain English
draft close summary
summarize reconciliation blockers
help user understand why period cannot close
suggest likely correction route
```

Forbidden:

```text
post journal
approve journal
close period
reopen period
override account code
invent accounting policy
invent tax treatment
force balanced entry by hallucination
```

Example:

```text
GPT-5.5 may suggest that "forklift repair invoice" likely maps to Repairs and Maintenance.

Accounting rules decide final account, tax code, cost center, and posting.
```

## 20. Access and Authority Gates

Protected accounting actions:

```text
create manual journal
approve journal
post manual journal
reverse journal
reopen period
close period
lock period
change chart of accounts
create account
deactivate account
post to restricted control account
override account mapping
override tax code
write off suspense
export ledger
view sensitive financial reports
```

Authority may depend on:

```text
role
entity
country
amount
account type
journal type
period status
risk level
board policy
finance policy
audit requirement
```

Step-up may be required for:

```text
period close
period reopen
manual journal above threshold
journal reversal
restricted account posting
ledger export
chart of accounts change
```

Authority failures must follow Master Access Governance and the Authority Failure Escalation + Supervisor Approval design where company policy allows.

## 21. Audit Requirements

Every accounting action must be auditable.

```text
audit_event_id
actor_id
action
journal_entry_id
ledger_posting_ids
account_ids
old_value_ref
new_value_ref
source_evidence_refs
authority_result
simulation_id
approval_refs
step_up_refs
timestamp
reason_code
period_id
company_id
legal_entity_id
country
currency
employee_visible_summary_optional
internal_summary
```

No raw bank details.

No secrets.

No silent edits.

## 22. Failure Branches

### Unbalanced journal

```text
Journal cannot post.
Reason: debits do not equal credits.
Action: create journal exception and ask owner to fix.
```

### Missing account mapping

```text
Supplier bill category cannot map to account.
Action: create mapping review task for Accounting.
```

### Closed period

```text
Journal date falls inside closed period.
Action: block posting or request controlled reopening.
```

### Missing tax code

```text
Transaction requires tax code.
Action: route to Tax/Accounting owner.
```

### Posting to restricted account

```text
Account is system-controlled.
Action: block manual posting unless elevated approval path passes.
```

### Period close blocker

```text
Close blocked because AP, bank, payroll, or inventory evidence is unresolved.
Action: show blocker list and owner responsible.
```

## 23. Required Logical Packets

Future logical packets:

```text
ChartOfAccountsPacket
AccountCodePacket
AccountMappingPacket
AccountingEventPacket
JournalEntryPacket
JournalLinePacket
JournalPostingValidationPacket
LedgerPostingPacket
TrialBalancePacket
AccountingPeriodPacket
PeriodCloseChecklistPacket
PeriodCloseBlockerPacket
ManualJournalRequestPacket
JournalApprovalPacket
JournalReversalRequestPacket
SuspenseReviewPacket
AccountDimensionPacket
FinancialStatementBasePacket
AccountingAuditEvidencePacket
```

Codex must later map these to repo equivalents.

Do not claim they currently exist.

## 24. Example End-To-End Flow: Supplier Bill Posting

Supplier bill arrives for forklift repairs.

```text
AP receives supplier bill.
AP validates supplier and invoice.
Tax owner determines GST/VAT treatment.
Accounting Event Fabric creates accounting event.
Accounting maps repair expense account.
Accounting drafts journal:
  Debit Repairs and Maintenance
  Debit GST/VAT Receivable if claimable
  Credit Accounts Payable
Posting validation passes.
Journal posts to General Ledger.
AP liability remains open until payment.
Audit links journal to supplier bill.
```

Selene says:

```text
I posted the forklift repair bill to Repairs and Maintenance and Accounts Payable. GST treatment was applied from the approved tax rule. The bill is now waiting for AP payment approval.
```

## 25. Example End-To-End Flow: Month-End Close

User says:

```text
Selene, close May.
```

Selene does:

```text
PH1.X classifies protected accounting action.
Access checks authority.
Accounting checks period status.
Selene runs close checklist.
Bank reconciliation incomplete.
Payroll journal posted.
AP has one unapproved bill.
Inventory adjustment pending.
Depreciation posted.
Trial balance balanced except pending blockers.
Period close blocked.
```

Selene says:

```text
May is almost ready, but I can't close it yet. Three items need attention: one unapproved supplier bill, one pending inventory adjustment, and two unreconciled bank transactions. I can route each item to the right owner now.
```

That is precise, human, and honest. No pretending everything is fine because someone clicked "close."

## 26. What Must Not Happen

```text
no ledger posting without source evidence
no unbalanced journal posting
no GPT-5.5 final journal posting
no silent edits to posted journals
no closed-period posting without authority
no period close with unresolved mandatory subledgers
no AP/AR control account manual chaos
no suspense account left unresolved indefinitely
no chart of accounts changes without audit
no tax code override without authority
no financial statements from draft journals
no client or adapter owning accounting truth
no implementation from this document alone
```

## 27. Future Simulation Targets

This document should later produce simulations such as:

```text
SIM_GL_001_supplier_bill_to_journal_posting
SIM_GL_002_pos_sale_to_revenue_tax_cogs_posting
SIM_GL_003_payroll_handoff_to_payroll_journal
SIM_GL_004_manual_journal_approval_and_posting
SIM_GL_005_journal_reversal
SIM_GL_006_period_close_with_blockers
SIM_GL_007_period_reopen_with_authority
SIM_GL_008_suspense_account_resolution
SIM_GL_009_trial_balance_generation
SIM_GL_010_closed_period_posting_blocked
```

## 28. Final Architecture Sentence

Selene General Ledger + Chart of Accounts + Journals + Period Close is the accounting book engine: it receives financial evidence from Selene's operational systems, maps each event to governed account codes, drafts or posts balanced double-entry journals, maintains the official ledger, controls accounting periods, produces trial balance and reporting foundations, blocks uncertain or unauthorized postings, and closes periods only when every required subledger, reconciliation, journal, tax, asset, inventory, payroll, AP, AR, and audit gate proves the books are ready.

## Related Addendum

Chart governance, account/category merging, lawful tax-optimization study, and error-prevention enhancements are defined in SELENE_GENERAL_LEDGER_CHART_GOVERNANCE_ACCOUNT_MERGE_TAX_OPTIMIZATION_ADDENDUM.md and must be read with this document.
