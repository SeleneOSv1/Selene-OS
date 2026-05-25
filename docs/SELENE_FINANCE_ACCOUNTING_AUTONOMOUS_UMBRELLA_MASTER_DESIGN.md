# Selene Finance + Accounting Autonomous Umbrella Master Design

```text
DOCUMENT TYPE:
MASTER DESIGN / FINANCE + ACCOUNTING AUTONOMOUS UMBRELLA

STATUS:
MASTER_DESIGN
NOT_RUNTIME_IMPLEMENTATION
PENDING_GRAND_ARCHITECTURE_RECONCILIATION

PURPOSE:
Define Selene's autonomous, real-time, international Finance + Accounting heart: the umbrella system that connects sales, POS, purchases, payroll, banking, AP, AR, budgets, credit cards, assets, inventory, tax, payments, cashflow, and financial approvals into one evidence-driven accounting nervous system.
```

## 0. Authority and Scope

AGENTS.md controls execution.

This is a docs-only master design.

No runtime code was changed.

This document does not authorize implementation.

This document is part of Finance/Accounting Design Batch 1 only. It does not create Accounts Payable, Accounts Receivable, Banking, Credit Card, Budget, Cashflow, Assets, Inventory, Tax, Multi-Entity, Dividends, or Evidence Fabric implementation documents. Those future documents remain pending controlled JD batches.

Current repo truth does not prove a complete runtime Finance/Accounting engine. This document defines future architecture and must not be read as a claim of implemented runtime behavior.

Future implementation requires Grand Architecture Reconciliation, explicit build instruction, approved file scope, repo-truth activation, tests, backend evidence, provider proof where relevant, Access/Governance proof, PH1.WRITE proof, audit proof, and JD live acceptance where visible.

## 1. Executive Target

Selene Accounting must not behave like old accounting software.

Old accounting:

```text
humans enter transactions
humans upload invoices
humans chase debtors
humans approve payments
humans reconcile bank feeds
humans classify expenses
humans prepare reports
humans fix mistakes later
```

Selene Accounting:

```text
business events create accounting evidence automatically
Selene classifies the evidence
Selene posts clean entries where rules are certain
Selene asks humans only for approval, missing evidence, or exceptions
Selene reconciles bank and credit card feeds automatically
Selene watches cashflow before payments are made
Selene chases debtors automatically
Selene schedules payments automatically
Selene escalates approvals automatically
Selene keeps the business running inside budget
```

The target is:

```text
real-time accounting
multi-country accounting
multi-entity accounting
autonomous bookkeeping
automatic AP and AR
automatic debtor chasing
automatic scheduled payments
automatic cashflow checks
automatic bank and credit card reconciliation
automatic tax/GST/VAT evidence
automatic payroll accounting
automatic asset and depreciation handling
automatic budget monitoring
human review only for exceptions and protected approvals
```

Plain human version: Selene should not wait for someone to type financial history into accounting. She should record the business as it happens.

## 2. Master Law

```text
Every financial event inside Selene must create accounting evidence.

Every accounting entry must trace back to source evidence.

Every payment must trace back to approval, authority, simulation, payment proof, and reconciliation.

Every uncertain financial event becomes an exception, not a silent guess.

Every country-specific rule must be source-backed, company-approved, or accountant-approved.

OpenAI / GPT-5.5 may explain and propose.

Deterministic Accounting, Finance, Tax, Access, and Payment owners decide what is posted, paid, reported, and audited.
```

The umbrella law is evidence first, owner truth second, automation third, and human approval wherever money, authority, compliance, or uncertainty requires it.

## 3. Repo-Truth Foundation

Current repo truth does not prove a complete runtime Finance/Accounting engine yet.

So this document is not claiming Selene already has:

```text
General Ledger
Chart of Accounts
Journals
Accounts Payable
Accounts Receivable
Bank payment rails
Bank reconciliation
Credit card reconciliation
Tax/GST/VAT engine
Fixed assets
Depreciation
Budgeting
Cashflow forecasting
Multi-entity consolidation
```

This document defines the future architecture.

Important naming traps must be avoided:

```text
RealtimeFinance must not be mistaken for internal company accounting unless repo truth proves it.
PH1.COMP must not be mistaken for Compensation or Finance truth.
AP must be disambiguated:
  Access AP = Approval Policy
  Finance AP = Accounts Payable
storage ledgers must not be mistaken for General Ledger unless repo truth proves it.
```

No fake accounting. No "ledger vibes." No financial cosplay.

## 4. High-Level Owner Split

Selene Finance + Accounting is an umbrella.

It coordinates many owners. It must not become one monster engine that owns everything.

### Accounting owns

```text
Chart of Accounts
General Ledger
journal entries
period close
financial statements
ledger posting
reversals
reconciliation accounting
audit trail for books
```

### Finance / Budget owns

```text
budgets
cashflow
payment priority
spend limits
profitability
forecasting
cost centers
board approval
financial governance
```

### Accounts Payable owns

```text
supplier bills
contractor invoices
purchase invoices
scheduled payments
installments
lease payments
payment approvals
outgoing liabilities
```

Accounts Payable must not be confused with Access Approval Policy.

### Accounts Receivable owns

```text
customer invoices
receipts
debtor chasing
collections
credit notes
refunds
payment terms
30 / 60 / 90 day aging
```

### Banking / Payments owns

```text
bank connections
payment rails
bank transfers
payment confirmations
bank feeds
credit card feeds
reconciliation
payment failures
```

### Credit Cards / Employee Spend owns

```text
company cards
employee card budgets
card limits
merchant classification
receipts
spend exceptions
card reconciliation
expense claims
reimbursements
```

### Assets owns

```text
fixed assets
asset register
capitalization
depreciation
asset disposal
business/private use
country depreciation rules
claimable asset treatment
```

### Inventory / COGS owns

```text
stock value
inventory movements
cost of goods sold
stock write-offs
stock adjustments
warehouse inventory accounting handoff
```

### Tax / Compliance owns

```text
GST
VAT
sales tax
withholding tax
payroll tax handoff
country tax rules
reporting periods
tax authority reporting
source-backed compliance rules
```

### Payroll owns

```text
employee pay calculation
payrun
payslip
employee pay instruction
payroll liabilities handoff
```

Payroll calculates employee pay. Accounting posts books and validates ledger effects. Banking/Payment providers execute transfers only through governed proof.

### Procurement / Purchasing owns

```text
purchase requests
purchase orders
supplier selection
goods receipt
purchase approval
```

### POS / Sales owns

```text
sale event
refund event
discount
customer payment event
sales tax event
sales commission trigger evidence
```

### Access / Governance owns

```text
who can view financial data
who can approve payments
who can post journals
who can approve budgets
who can approve supplier bills
who can approve refunds
who can override tax treatment
who can export financial records
```

### PH1.D / GPT-5.5 owns proposal help only

```text
classify expense category candidates
explain reports
draft debtor reminders
summarize invoices
explain cashflow warnings
explain budget variances
draft board summaries
```

It must not:

```text
post journals
approve payments
execute transfers
invent tax law
override budgets
change accounting rules
```

### PH1.WRITE owns final explanation

```text
finance explanations
approval summaries
debtor reminders
supplier messages
budget warnings
payment confirmations
board summaries
```

## 5. Core Architecture: Accounting Event Fabric

Every meaningful business action creates an event.

```text
POS sale
supplier bill received
purchase order approved
customer invoice issued
customer payment received
bank payment confirmed
payroll committed
contractor milestone approved
credit card transaction received
asset purchased
inventory sold
stock written off
lease payment due
tax payment due
budget exceeded
refund approved
```

Each event becomes:

```text
AccountingEventPacket
```

Suggested future logical fields:

```text
event_id
event_type
source_engine
tenant_id
legal_entity_id
company_id
country
region
currency
event_date
accounting_period
counterparty_ref
source_document_ref
amount_gross
amount_net
tax_amount
tax_code_candidate
account_code_candidate
cost_center_ref
department_ref
project_ref
location_ref
payment_terms_ref
due_date
evidence_refs
trust_status
requires_review
audit_ref
```

The Accounting Event Fabric decides:

```text
Is this financial?
Which owner should process it?
Can it be auto-posted?
Does it need approval?
Does it need tax review?
Does it create AP?
Does it create AR?
Does it create an asset?
Does it affect inventory?
Does it affect payroll?
Does it affect budget?
Does it require payment?
```

This is the heart of autonomous accounting. It is future logical architecture and must later be mapped to repo contracts before implementation.

## 6. Real-Time Double-Entry Posting

Accounting must use proper double-entry logic.

Example: customer sale through POS.

```text
Debit: Bank / Card Clearing
Credit: Sales Revenue
Credit: GST/VAT/Sales Tax Payable
Debit: Cost of Goods Sold
Credit: Inventory
```

Example: supplier bill for stock.

```text
Debit: Inventory
Debit: GST/VAT Receivable if claimable
Credit: Accounts Payable
```

Example: payroll committed.

```text
Debit: Wage Expense
Debit: Employer Contribution Expense
Credit: Payroll Payable
Credit: Tax Withholding Payable
Credit: Super/Pension/CPF Payable
```

Example: lease installment due.

```text
Debit: Lease Expense or Lease Liability / Interest depending policy
Credit: Accounts Payable or Bank
```

Selene can auto-post only when:

```text
source evidence is strong
rule is known
tax treatment is known
account mapping is known
approval is not required or already passed
period is open
no conflict exists
```

Otherwise:

```text
draft journal
review required
exception created
```

## 7. International Accounting Foundation

Selene must support global companies from the beginning.

Required dimensions:

```text
legal_entity_id
country
region/state/province
tax jurisdiction
functional currency
transaction currency
reporting currency
fiscal year
accounting period
local chart of accounts
group chart of accounts
local tax rule pack
local depreciation rule pack
local payroll accounting rule pack
local invoice rules
local reporting requirements
```

Selene must support:

```text
multi-company
multi-country
multi-currency
multi-tax rules
multi-entity consolidation
intercompany invoices
intercompany loans
FX revaluation
local statutory reporting
group management reporting
country-specific depreciation
country-specific claimable expenses
country-specific payment methods
country-specific payroll liabilities
```

Example:

```text
ABC Wines Australia reports in AUD and GST.
ABC Wines Singapore reports in SGD and GST.
ABC Wines UK reports in GBP and VAT.
Parent group consolidates in AUD.
```

Selene handles local books and group reporting separately.

No single-country accounting brain.

## 8. Automatic AP / AR Terms: 30 / 60 / 90 Days

Selene must manage both money going out and money coming in.

### Accounts Payable terms

```text
supplier bill due in 30 days
contractor invoice due in 14 days
lease payment due monthly
loan repayment due quarterly
tax payment due by statutory date
```

Selene must:

```text
read due date
store payment terms
schedule payment
check cashflow before due date
route approval before payment
warn if payment risk exists
pay through approved bank/payment rails
reconcile payment after bank confirmation
```

### Accounts Receivable terms

```text
customer invoice due in 30 days
customer overdue at 31 days
aging bucket: 30 / 60 / 90 / 120+
collection escalation
```

Selene must:

```text
send invoice
track due date
send friendly reminders
increase firmness over time
escalate to manager/collections when needed
record customer promises
match bank receipt
close invoice when paid
```

Customer reminder style:

```text
Day 3 before due:
"Just a friendly reminder that invoice INV-1024 is due soon."

Day 1 overdue:
"Invoice INV-1024 is now overdue. Could you please arrange payment or let us know if there's an issue?"

30+ days:
"This invoice is now 30 days overdue. Please arrange payment urgently or contact us today to resolve it."
```

Production Selene should be firm, professional, persistent, and PH1.WRITE-governed. AR source truth must control whether a reminder can be sent.

## 9. Selene-to-Selene Company Communication

If both companies use Selene, invoices and payment requests should move system-to-system.

No need for email attachments if both sides support Selene financial exchange.

Flow:

```text
Company A Selene creates invoice.
Company A Selene sends invoice packet to Company B Selene.
Company B Selene receives and validates invoice.
Company B Selene matches purchase order or contract.
Company B Selene routes approval.
Company B Selene schedules payment.
Company A Selene tracks receivable.
Both Selenes exchange payment status.
Bank confirmation closes the loop.
```

Future packet:

```text
SeleneToSeleneInvoicePacket
- sender_company_id
- receiver_company_id
- invoice_id
- invoice_lines
- tax_details
- payment_terms
- due_date
- bank/payment instructions
- evidence_refs
- signature/proof
- delivery_status
```

This should support:

```text
invoice delivery
bill receipt
payment request
payment status
remittance advice
dispute message
credit note
statement
```

Important:

```text
Selene-to-Selene delivery uses BCAST/DELIVERY or future secure connector protocol.
Accounting owns source truth.
Delivery does not decide financial truth.
```

## 10. Installments, Lease Payments, Recurring Payments

Selene must handle automatic scheduled payments.

Examples:

```text
monthly lease payment
vehicle loan installment
equipment financing
software subscription
rent
insurance premium
tax installment
supplier payment plan
employee salary advance repayment
customer payment plan
```

Recurring payment setup:

```text
payment_schedule_id
counterparty_id
contract_ref
amount
currency
frequency
start_date
end_date
next_due_date
payment_method
approval_required
cashflow_check_required
auto_pay_allowed
bank_provider_ref
audit_ref
```

Before each payment, Selene checks:

```text
is payment due?
is invoice/contract valid?
is cash available?
is approval required?
has budget changed?
is bank account valid?
is payment already made?
is the accounting period open?
```

If all clear:

```text
prepare payment
route approval if needed
send via bank/payment provider
record confirmation
post accounting entry
reconcile
```

If not clear:

```text
PaymentExceptionPacket
```

## 11. Cashflow Before Payment

Selene should never blindly pay bills if cashflow is tight.

Before payment, Selene checks:

```text
current bank balance
expected incoming receipts
upcoming payroll
upcoming tax payments
scheduled supplier payments
lease/loan installments
minimum cash reserve
budget priority
payment terms
penalty/interest risk
supplier criticality
```

Cashflow result:

```text
pay_now
schedule_for_due_date
delay_if_allowed
partial_payment
requires_finance_review
cash_shortfall_warning
```

Example:

```text
"Cashflow is tight this week. Payroll and tax payments are due before the supplier invoice. I recommend delaying Supplier A by 5 days if terms allow, and paying the lease on time because late fees apply."
```

This is Selene acting like a real finance operator: checking the bank, upcoming obligations, reserves, terms, and risk before money moves.

## 12. Multi-Authority Payment Approval

Payments may require more than one approver.

Examples:

```text
CEO + CFO
CFO + Finance Manager
2 of 3 directors
board approval
chairman approval
department manager + finance
```

Approval matrix fields:

```text
amount_threshold
payment_type
country
entity
department
vendor_risk
new_bank_account
budget_status
number_of_approvers_required
required_roles
step_up_required
simulation_required
audit_required
```

Example:

```text
Payment over $100,000 requires CEO and CFO.
Both Harry CEO and Bob CFO must approve.
Both must pass Face ID/fingerprint/passcode step-up.
Only after both approvals does Selene send payment through bank API.
```

Approval flow:

```text
Selene prepares payment.
Access resolves required approvers.
PH1.WRITE prepares approval summary.
PH1.BCAST/DELIVERY sends approval requests.
Harry approves with Face ID.
Bob approves with Face ID.
Access creates approval proof.
Payment provider executes.
Bank confirms.
Accounting posts and reconciles.
```

No approval, no payment.

No stale chat approval, no implied permission, no payment without authority, simulation, step-up where required, provider proof, and audit.

## 13. Bank and Credit Card Connectivity

Banks and credit cards must connect directly to Selene where supported.

Selene must ingest:

```text
bank balances
bank transactions
card transactions
payment confirmations
merchant data
fees
chargebacks
refunds
interest
loan repayments
currency conversions
```

Selene reconciles:

```text
bank transactions to invoices
bank payments to supplier bills
payroll payments to payruns
card transactions to expense claims
merchant fees to POS settlements
loan repayments to schedules
tax payments to liabilities
```

Credit card management includes:

```text
company cards
employee cards
spend budgets
merchant category controls
receipt requirement
missing receipt reminders
suspicious spend detection
personal spend flag
budget limit warnings
card reconciliation
```

Example:

```text
Tom spends $180 on fuel.
Selene sees credit card transaction.
Selene asks Tom for receipt and business purpose.
Selene matches vehicle/project.
Selene classifies expense.
Selene posts accounting.
Selene reconciles card statement.
```

## 14. Annual Budget Creation and Board Approval

Selene should help prepare annual budgets.

Inputs:

```text
last year actuals
current year forecast
sales forecast
payroll forecast
contractor cost
inventory forecast
asset purchases
leases
tax payments
department plans
growth targets
cash reserve policy
country/entity rules
```

Selene prepares:

```text
draft annual budget
department budgets
cost center budgets
capital expenditure budget
payroll budget
marketing budget
inventory budget
cashflow forecast
profit forecast
risk notes
```

Then routes approval:

```text
management review
CFO review
CEO review
board approval
chairman approval if required
```

After approval, Selene runs the company against budget.

```text
budget becomes active
spend checks use active budget
over-budget actions require approval
cashflow forecast uses budget
monthly variance reports generated
```

## 15. Budget Increase / Overrun Approval

If budget is exceeded, Selene must not just block blindly.

Flow:

```text
spend request exceeds budget
Selene identifies budget line
Selene calculates variance
Selene explains reason
Selene checks urgency
Selene proposes options
Selene routes increase request
board/management approves or denies
budget updated if approved
audit
```

Example:

```text
"Warehouse maintenance is $8,200 over budget because forklift repairs were higher than forecast. I recommend increasing the quarterly maintenance budget by $10,000. This requires CFO and board approval."
```

This is Finance/Budget, not AP alone.

## 16. Financial Dashboards and Real-Time Health

Selene should always know:

```text
cash today
cash forecast
AP due this week
AR overdue
payroll due
tax due
budget variance
profit margin
top debtors
top suppliers
stock value
credit card spend
unmatched bank transactions
payment approval bottlenecks
```

Selene can answer:

```text
Can we afford payroll?
Who owes us money?
Who do we owe money to?
What payments are due this week?
Which customers are late?
Which department is over budget?
What is our cash runway?
Can we buy this vehicle?
Should we hire or use contractors?
```

## 17. Automation Principles

Selene should automate:

```text
invoice creation
invoice sending
debtor reminders
payment scheduling
installment payments
lease payments
bank reconciliation
credit card reconciliation
receipt chasing
expense classification
tax code suggestions
journal drafting
budget variance alerts
approval routing
board summaries
cashflow warnings
asset depreciation
payroll accounting
contractor payment checks
```

Selene should not automate without approval:

```text
high-value payments
bank account changes
new supplier bank details
large refunds
tax submissions
period close
journal reversals
budget increases
asset write-offs
salary payments without payroll approval
```

## 18. Access and Authority Law

Financial authority is strict.

Actions requiring access/authority:

```text
view bank account
view financial reports
view supplier bank details
approve payment
send payment
approve budget
change budget
post journal
reverse journal
approve refund
approve write-off
approve new supplier
approve contractor invoice
approve tax submission
export financial data
```

Authority can depend on:

```text
role
amount
country
entity
department
payment type
vendor risk
budget status
bank account
dual approval requirement
board threshold
```

Step-up verification required for:

```text
payment approval
bank transfer
supplier bank changes
large refunds
budget override
tax submission
journal reversal
high-risk export
```

Authority failures must consume Master Access escalation law where policy allows. Finance/Accounting must not invent local authority-failure behavior.

## 19. PH1.D and PH1.WRITE

Selene can use GPT-5.5 heavily for finance communication.

Allowed:

```text
explain budget variance
draft debtor reminder
summarize invoice
classify expense candidate
explain cashflow issue
draft board summary
draft payment approval summary
explain reconciliation mismatch
explain why payment cannot proceed
```

Not allowed:

```text
approve payment
execute bank transfer
post final journal
invent tax rule
override budget
decide final account classification for protected transactions
expose private finance data
```

PH1.WRITE finalizes all human-facing wording.

Finance/accounting truth comes from deterministic owners.

## 20. Audit and Traceability

Every financial action must be traceable.

Audit must record:

```text
who initiated
who approved
what changed
old value
new value
source evidence
simulation id
authority result
step-up result
payment provider ref
bank confirmation ref
journal ref
invoice ref
bill ref
tax rule ref
timestamp
company/entity
country
currency
reason
```

No raw secrets in logs.

No bank credentials in audit.

No hidden edits.

## 21. Failure Handling

Common failures:

```text
cash unavailable
approval missing
approver timeout
bank API failure
partial payment failure
wrong bank details detected
invoice duplicate
tax rule missing
budget exceeded
period closed
credit card receipt missing
customer disputes invoice
supplier invoice mismatch
contractor overrun
bank reconciliation mismatch
foreign exchange rate missing
Selene-to-Selene delivery failed
```

Every failure creates an exception packet and a human-safe explanation.

Example:

```text
"I can't release this supplier payment yet. The invoice is valid, but the payment would drop cash below the company's minimum reserve. I've prepared options for Finance."
```

## 22. Executive / Employee Card Personal Spend Recovery Law

Selene must review company credit cards, executive cards, and employee cards.

Selene must separate business charges from personal charges, request receipts and business purpose, classify suspicious or personal spend, and recover personal or non-business charges through the correct company policy.

Recovery options may include:

```text
payroll deduction where lawful and approved
employee repayment request
director/shareholder loan account
reimbursement reversal
expense rejection
manager/finance review
```

Selene must retain all old records and evidence:

```text
original card transaction
receipt request
employee response
business-purpose explanation
classification result
review result
recovery method
repayment or deduction evidence
audit refs
```

PH1.D may help summarize the transaction and draft the employee question. Finance, Payroll, Accounting, Access, and Audit owners decide the actual recovery path.

## 23. Employee Advance / Repayment Law

If an employee receives money in advance, Selene must track:

```text
advance_id
employee_id
approved_amount
paid_amount
approval_ref
payment_ref
repayment_schedule
payroll_deduction_timing
remaining_balance
minimum_net_pay protection where lawful/policy-required
employee-visible explanation
audit_ref
```

If an employee asks to delay repayment, Selene must check:

```text
company policy
jurisdiction rule
payroll cutoff
approval requirements
cash impact
remaining advance balance
employee-visible repayment consequence
```

Selene must not silently move advance repayment deductions. Payroll owns payroll deduction application. Finance/Accounting owns accounting impact. Access/Governance owns approval requirements.

## 24. Minimum Cash Reserve Law

Authorized executives or board-approved users may set a minimum cash reserve rule.

Example:

```text
"Selene, our reserve cash should now be $12 million. Do not allow normal payments to take us below that."
```

Selene must apply the reserve rule to:

```text
payment scheduling
AP payment approval
payroll timing
contractor payments
lease payments
cashflow forecasting
budget overrun review
payment priority recommendations
```

Emergency override may be allowed only through the required approval path.

Minimum cash reserve must not be bypassed by AP, Payroll, Banking, Adapter, Desktop, iPhone, PH1.D, or local convenience logic.

## 25. Net Profit Floor / Profitability Governance Law

Authorized management or board may set minimum net profit targets.

Example:

```text
"Net profit must not fall below 8%."
```

Selene must monitor:

```text
actual net profit
forecast net profit
monthly trend
department contribution
discretionary spend impact
budget variance
cashflow impact
margin risk
```

When actual or forecast profit falls below target, Selene must:

```text
warn management
identify drivers
restrict discretionary spend where policy allows
escalate budget/spend exceptions to management or board
prepare options
audit decisions
```

Profitability governance belongs to Finance/Budget. Accounting provides ledger truth. PH1.D may help explain variance but must not override profitability policy.

## 26. Equity / Dividend Distribution Boundary

Selene must support future shareholder distribution logic, including:

```text
dividend policy
retained earnings
shareholder classes
solvency/cashflow tests
approval requirements
tax treatment
payment execution
audit
```

Dividend logic belongs in a dedicated future Equity + Shareholder Distributions + Dividends document.

Example:

```text
"Distribute 30% of net profits to shareholders. Class A receives X, Class B receives Y."
```

Selene must calculate, prepare, route approval, and pay only through legal, accounting, tax, access, payment, and audit gates.

This batch does not write the dividend document. It only preserves the umbrella boundary.

## 27. What Must Not Happen

```text
no accounting event without evidence
no payment without authority
no bank API call without payment-owner proof
no tax rule invented by GPT-5.5
no AP confused with Access Approval Policy
no credit card spend hidden from reconciliation
no debtor reminders sent without AR source truth
no invoice delivery bypassing Delivery protocol
no budget increase without approval
no journal posting from LLM suggestion alone
no contractor paid through Payroll by default
no supplier bank detail change without step-up
no old financial data overwritten without history
no multi-country company forced into one-country rules
no minimum cash reserve bypass
no net profit floor bypass where policy applies
no personal card spend recovery without evidence and lawful policy
no employee advance repayment schedule change without policy, approval, and payroll cutoff check
no shareholder distribution without legal, accounting, tax, payment, approval, and audit gates
no implementation from this document alone
```

## 28. Batch 1 Future Document Map

This umbrella controls the following future finance/accounting documents. These future documents are not written in this batch.

```text
3. Accounts Payable + Supplier Bills + Installments + Scheduled Payments
4. Accounts Receivable + Invoices + Debtor Chasing + Collections
5. Banking + Payment Rails + Reconciliation
6. Credit Cards + Employee Spend + Reimbursements
7. Budgeting + Spend Control + Board Approval
8. Cashflow Forecasting + Payment Priority Intelligence
9. Assets + Depreciation + Claimable Expense Rules
10. Inventory + COGS + Stock Accounting Handoff
11. Tax / GST / VAT / Country Compliance
12. Multi-Entity + Multi-Currency + Consolidation
13. Finance/Accounting Evidence Fabric + Codex Readiness Layer
14. Equity + Shareholder Distributions + Dividends
```

Future batch rule:

```text
Do not pre-write future finance/accounting documents from this batch.
Do not infer detailed AP, AR, Banking, Credit Card, Budget, Cashflow, Assets, Inventory, Tax, Multi-Entity, Dividends, or Evidence Fabric rules beyond the source material preserved here.
Those documents require JD-provided source material in controlled future batches.
```

## 29. Example: Complete Autonomous Money Flow

User says:

```text
Buy a new delivery van.
```

Selene does:

```text
checks user authority
checks budget
checks cashflow
checks asset policy
checks country depreciation rules
checks GST/VAT claimability
prepares purchase order
routes approval
receives supplier invoice
creates AP bill
creates fixed asset draft
prepares payment schedule
routes payment approval
sends payment through bank provider
records bank confirmation
posts journal
starts depreciation schedule
reconciles payment
updates budget and forecast
```

User sees:

```text
"The delivery van purchase is ready for approval. It fits the vehicle budget, GST appears claimable under the current rule pack, and depreciation will start when the van is placed in service. This payment requires CFO approval before I send it to the bank."
```

That is Selene Accounting: not a form, not a report, but a governed financial operator.

## 30. Final Architecture Sentence

Selene Finance + Accounting Autonomous Umbrella defines Selene's real-time international financial heart: every sale, invoice, bill, payroll, contractor payment, lease installment, credit card charge, purchase order, asset purchase, tax event, bank movement, budget decision, debtor reminder, and payment approval becomes structured accounting evidence that flows through the correct finance, accounting, AP, AR, banking, tax, access, and audit owners so Selene can run money operations continuously, globally, safely, and with humans involved only where approval, exception, or judgment is truly required.
