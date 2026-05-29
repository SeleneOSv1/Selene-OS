# Finance / Accounting Batch 6 — Documents 11–12 Bank Reconciliation, Treasury, Cashflow + Working Capital

```text
DOCUMENT TYPE:
FINANCE / ACCOUNTING MASTER DESIGN BATCH

STATUS:
FUTURE_CANONICAL_ARCHITECTURE
NOT_RUNTIME_IMPLEMENTATION
PENDING_REPO_TRUTH_ACTIVATION
PENDING_SIMULATION_MAPPING
```

## Batch Contents

- Finance / Accounting Document 11 — Bank Reconciliation + Treasury Control
- Finance / Accounting Document 12 — Cashflow Forecasting + Working Capital Optimization
- Supplier early / urgent payment and early-payment discount logic
- Selene Autonomous Trigger + Event Fabric / Automation Event Trigger Fabric content

This batch mechanically consolidates the previously accepted standalone source documents below. Architecture content is preserved verbatim between source-file markers.

<!-- BEGIN_SOURCE_FILE: docs/SELENE_FINANCE_ACCOUNTING_DOCUMENT_11_BANK_RECONCILIATION_TREASURY_CONTROL_MASTER_DESIGN.md -->
# Finance / Accounting Document 11 — Selene Bank Reconciliation + Treasury Control Engine

```text
DOCUMENT TYPE:
FINANCE / ACCOUNTING MASTER DESIGN

DOCUMENT NUMBER:
11

ENGINE:
PH1.BANKREC / PH1.TREASURY

FULL NAME:
Selene Bank Reconciliation, Treasury Control, Cash Position, Bank Feed, Payment Proof, and Liquidity Verification Engine

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

This document defines future canonical architecture for PH1.BANKREC / PH1.TREASURY. Repo-truth activation, simulation mapping, owner mapping, tests, and approved implementation slices must happen later before runtime behavior can be claimed.

## 1. Purpose

Selene Bank Reconciliation + Treasury Control Engine owns the truth between what Selene thinks happened financially and what the bank actually shows happened.

It answers:

```text
What is the real bank balance?
Which payments cleared?
Which customer receipts arrived?
Which supplier payments failed?
Which bank transactions are unmatched?
Which bank fees, interest, FX, chargebacks, or reversals occurred?
Which transactions are suspicious?
Which accounts need reconciliation?
Is the cash position safe?
Can Selene trust the current cashflow forecast?
```

This engine is the financial reality mirror.

Accounting may record the expected transaction.

AP may schedule supplier payment.

AR may expect customer receipt.

POS may record sales.

E-commerce may show payouts.

Payroll may show salary payment instructions.

But the bank says what actually moved.

Selene must reconcile the two.

Expected payment is not the same as money arrived.

## 2. Core Selene Law

```text
No bank balance is trusted without source proof.
No payment is treated as cleared without bank/provider confirmation.
No receipt is treated as fully reconciled until matched.
No unexplained bank movement is ignored.
No cashflow forecast is trusted if bank reconciliation is stale.
Routine matches are automatic.
Exceptions are escalated only when Selene cannot resolve them safely.
```

Selene must reduce human work.

Humans should not manually tick every clean bank line.

Selene should:

```text
import bank data
match automatically
explain exceptions
post routine fees if policy allows through Accounting workflow
flag suspicious activity
update cashflow
prepare close packs
ask humans only for unresolved exceptions
```

## 3. Engine Boundary

### 3.1 PH1.BANKREC / PH1.TREASURY Owns

```text
bank account register
bank feed ingestion
bank statement import
bank transaction normalization
payment confirmation matching
customer receipt matching
supplier payment matching
payroll payment matching
tax payment matching
POS settlement matching
e-commerce payout matching
bank fee detection
interest detection
FX difference detection
chargeback/reversal detection
unmatched transaction worklist
daily cash position
liquidity verification
bank reconciliation status
treasury movement tracking
cash account control reporting
reconciliation audit pack
```

### 3.2 PH1.BANKREC / PH1.TREASURY Does Not Own

```text
supplier invoice validation
customer invoice creation
payroll calculation
tax law truth
payment authority
ledger posting final approval
product/inventory truth
bank credential ownership
```

### 3.3 Correct Owner Split

```text
AP says what suppliers should be paid.
AR says what customers owe and paid.
Payroll says what employees should receive.
Tax says what tax should be paid.
Payment Engine sends payment instructions.
BankRec/Treasury proves what cleared at the bank.
Accounting posts final books.
Finance/Cashflow uses reconciled cash truth.
Access/Authority controls protected actions.
Audit records proof.
```

BankRec is not the bank.

Treasury is not AP.

Accounting is not bank feed import.

Each owner stays in its lane.

## 4. Bank Account Register

Selene must maintain a company bank account register.

Each bank account record includes:

```text
bank_account_id
legal_entity_id
bank_name
account_name
masked_account_number
currency
country
branch / routing data
account_type
operating account / payroll / tax / savings / loan / escrow / merchant clearing
linked payment provider
linked bank feed status
authorized users
approval rules
statement cycle
last_feed_sync
last_reconciliation_date
reconciliation_status
cashflow_inclusion_flag
audit_ref
```

Bank account statuses:

```text
Draft
PendingVerification
Active
FeedConnected
FeedDisconnected
Suspended
Closed
Archived
```

Selene should say:

```text
Your operating account is connected and last reconciled this morning. The payroll account has not synced for two days.
```

## 5. Bank Feed And Statement Intake

Selene accepts bank data from:

```text
bank feed integration
open banking connection
uploaded bank statement
CSV file
PDF statement
payment provider payout report
merchant settlement report
bank API
manual statement upload
```

Selene extracts:

```text
transaction date
value date
description
amount
currency
bank reference
counterparty
payment reference
fees
balance after transaction
transaction type
source account
```

Selene normalizes messy descriptions.

Example:

```text
"STRIPE PAYOUT 83920"
"Stripe settlement"
"STP PYOUT 83920"
```

Selene should recognize these as likely payment-provider settlement patterns.

GPT-5.5 can help interpret messy descriptions, but deterministic matching rules confirm truth.

## 6. Reconciliation Matching Model

Selene uses layered matching.

### 6.1 Exact Match

```text
same amount
same reference
same date or allowed date window
same counterparty
known payment/receipt ID
```

Exact matches may auto-match.

### 6.2 Strong Match

```text
same amount
near date
same supplier/customer/payment provider
reference partially matches
```

Strong matches may auto-match if policy allows.

### 6.3 Probable Match

```text
amount matches
description resembles customer/supplier
date within window
no competing match
```

Selene proposes. Review may be required depending risk.

### 6.4 Weak Match

```text
ambiguous description
multiple possible invoices/payments
partial amount
foreign currency
missing reference
```

Weak matches go to exception worklist.

### 6.5 No Match

```text
unrecognized bank movement
no AP/AR/payroll/tax/payment-provider record
```

Selene opens an unmatched transaction case.

## 7. Bank Reconciliation Lifecycle

```text
BankDataReceived
Normalized
MatchingStarted
AutoMatched
PartiallyMatched
UnmatchedItemsDetected
ExceptionReviewRequired
AdjustmentProposed
AdjustmentApprovedOrAutoApplied
ReconciliationBalanced
ReconciliationClosed
Archived
```

Routine path:

```text
bank feed imports
→ Selene normalizes transactions
→ Selene matches to AP/AR/POS/e-commerce/payroll/tax
→ Selene posts or prepares routine adjustments through Accounting workflow
→ reconciliation closes
```

Exception path:

```text
bank feed imports
→ Selene finds unmatched transaction
→ Selene investigates
→ Selene explains the likely source
→ Selene asks only for missing decision/proof
```

No human reviews clean matches.

No one should manually reconcile hundreds of clean provider lines because the system failed to match them.

## 8. Customer Receipt Matching

Selene matches incoming bank transactions to:

```text
customer invoice
customer receipt
POS settlement
e-commerce payout
B2B account payment
deposit
partial payment
overpayment
underpayment
payment plan
bad debt recovery
```

### 8.1 Matching Rules

```text
customer reference
invoice number
amount
payment link reference
bank remittance
customer name
payment provider ID
```

### 8.2 Receipt Outcomes

```text
FullyMatched
PartiallyMatched
Overpayment
Underpayment
UnknownCustomerReceipt
DuplicateReceipt
ChargebackRisk
```

Selene says:

```text
Customer ABC paid $4,900 against a $5,000 invoice. I’ve marked it as partial and opened an underpayment reason check.
```

Selene does not call a receipt fully paid because the number was close.

## 9. Supplier Payment Matching

Selene matches bank outflows to:

```text
supplier payment instruction
payment batch
supplier invoice
remittance
bank/provider confirmation
AP payment record
```

Outcomes:

```text
PaymentCleared
PaymentPending
PaymentFailed
AmountMismatch
DuplicatePaymentRisk
UnknownSupplierPayment
BankFeeAttached
FXDifferenceDetected
```

Selene says:

```text
Supplier payment batch B-104 cleared except for Supplier ABC, which failed at the bank. AP remains open for that invoice.
```

Selene must feed this back to:

```text
AP
Supplier Payment Engine
Accounting
Cashflow
Supplier communication
```

## 10. POS, E-Commerce, And Payment Provider Settlement Matching

Selene must reconcile settlements from:

```text
card terminals
POS
Stripe / PayPal / wallets / payment providers
marketplaces
B2B platform escrow
e-commerce checkout
gift cards / store credit
BNPL providers where applicable
```

Payment provider settlement is not always one-to-one.

It may include:

```text
gross sales
refunds
chargebacks
fees
tax
tips/gratuities
payout delay
reserve hold
currency conversion
net payout
```

Selene must split settlement:

```text
Dr Bank
Dr Fees
Dr Chargebacks / Refunds
Cr Payment Clearing / Customer Receipts
```

Accounting owns final posting.

BankRec prepares the proof.

Selene should say:

```text
Stripe paid $9,640 today. This reconciles to $10,000 gross sales, $210 refunds, and $150 fees.
```

## 11. Payroll Payment Matching

Selene matches payroll bank outflows to:

```text
approved payroll run
employee payment batch
payroll tax payment
super/pension/CPF remittance
salary advance recovery
final pay
```

Payroll sensitive data must remain restricted.

Outcomes:

```text
PayrollBatchCleared
PartialPayrollFailure
EmployeePaymentFailed
PayrollTaxPaymentCleared
RemittancePending
```

Selene may say to Finance:

```text
Payroll batch cleared successfully.
```

Selene must not expose salary details to unauthorized users.

## 12. Tax Payment Matching

Selene matches tax payments to:

```text
GST/VAT/sales tax return
payroll tax
withholding tax
income tax installment
customs/import duties
local taxes
```

Tax Engine owns tax truth.

BankRec proves payment movement.

Outcomes:

```text
TaxPaymentCleared
TaxPaymentPending
TaxPaymentFailed
TaxAmountMismatch
TaxReferenceMismatch
```

## 13. Bank Fees, Interest, And Adjustments

Selene detects routine bank items:

```text
bank fees
merchant fees
interest income
interest expense
loan repayment
FX gain/loss
chargeback
reversal
cash deposit
cash withdrawal
rounding difference
```

If policy allows, Selene can prepare or auto-post routine low-risk journals through Accounting workflow.

Selene says:

```text
The bank charged a $12 monthly account fee. This matches your routine fee rule, so I’ve prepared the accounting entry.
```

If unusual:

```text
This $4,200 bank fee is outside normal pattern. I’m holding it for review.
```

Routine low-risk items can follow policy. Unusual items require review.

## 14. Unmatched Transaction Worklist

Unmatched bank transactions are classified.

```text
UnknownCustomerReceipt
UnknownSupplierPayment
PossibleBankFee
PossibleLoanMovement
PossibleOwnerContribution
PossibleRefund
PossibleChargeback
PossibleFraud
PossibleDuplicate
PossibleInternalTransfer
```

Selene investigates using:

```text
bank description
amount
date
counterparty
customer/supplier records
recent invoices
payment batches
POS settlements
e-commerce payouts
payroll runs
tax schedules
loan schedules
recurring patterns
```

Selene says:

```text
This $2,500 receipt looks like Customer DEF based on amount and remittance text, but it lacks invoice reference. I recommend matching it to invoice INV-882.
```

If confidence is high and policy allows, Selene can auto-match.

If ambiguous, Selene asks.

## 15. Suspicious Bank Movement Detection

Selene flags:

```text
unknown large outflow
duplicate supplier payment
payment to unrecognized account
cash withdrawal outside pattern
payment after supplier bank change
unusual weekend transfer
foreign currency outflow
unmatched merchant payout
unexpected refund/chargeback spike
manual bank transfer outside payment engine
```

Action:

```text
hold reconciliation close
alert Finance/Authority
attach bank evidence
recommend investigation
```

Selene should say:

```text
I found a bank transfer that does not match any approved payment instruction. I’ve marked it high-risk and opened an investigation.
```

## 16. Daily Cash Position

Treasury must provide real cash truth.

Daily cash position includes:

```text
bank balances
cleared cash
pending payments
pending receipts
payment provider balances
merchant clearing
escrow balances
restricted cash
payroll account
tax account
loan accounts
currency balances
unreconciled movements
```

Selene should answer:

```text
As of this morning, cleared cash is $184,200. Scheduled supplier payments are $42,000, payroll reserve is $60,000, and unrestricted operating cash is $82,200.
```

Not:

```text
Bank balance: $184,200.
```

## 17. Liquidity And Treasury Control

Selene tracks:

```text
minimum cash buffer
payroll buffer
tax buffer
supplier payment commitments
debt repayments
loan covenants
cash concentration
account sweeps
short-term surplus
cash shortfall warnings
restricted vs unrestricted cash
```

Selene can recommend:

```text
delay flexible supplier payments within terms
move cash between accounts
keep payroll buffer protected
take early payment discount only if safe
fund tax account
draw/repay facility if configured
warn about covenant risk
```

Protected actions like bank transfers between accounts require authority and payment/banking controls.

Selene recommends.

Authorized execution follows rules.

## 18. Internal Transfers

Internal bank transfers must be tracked separately.

Examples:

```text
operating to payroll account
operating to tax account
store cash deposit to bank
merchant clearing to operating account
intercompany cash movement
foreign currency transfer
```

Selene matches both sides:

```text
outflow from source
inflow to destination
timing difference
bank fees
FX difference
```

If only one side appears:

```text
InternalTransferInTransit
```

Selene does not treat internal transfers as income or expense.

## 19. Multi-Currency Bank Reconciliation

Selene supports:

```text
transaction currency
bank account currency
functional currency
reporting currency
FX rate
bank conversion rate
realized FX gain/loss
unrealized FX revaluation
foreign bank fees
```

If invoice was in USD but bank paid AUD equivalent, Selene identifies:

```text
invoice amount
payment amount
FX difference
bank fee
settlement rate
```

Accounting posts FX.

BankRec proves movement and variance.

## 20. Bank Account Reconciliation Close

Each bank account gets a reconciliation status.

```text
NotStarted
InProgress
AutoMatched
ExceptionsOpen
ReadyForReview
Closed
Locked
Reopened
```

Close requirements:

```text
all bank lines imported
opening balance agrees
closing balance agrees
all transactions matched or classified
unmatched items below policy threshold or reviewed
bank fees/interest handled
payments in transit listed
deposits in transit listed
audit pack generated
```

Selene says:

```text
Operating account reconciliation is closed for May. Two deposits remain in transit and are listed in the close pack.
```

## 21. Accounting Handoff

BankRec sends Accounting:

```text
matched receipts
matched payments
bank fees
interest
FX differences
chargebacks
refunds
unmatched classifications
internal transfers
payment provider settlements
reconciliation close pack
```

Accounting owns journal posting and ledger truth.

BankRec provides proof and proposed entries.

## 22. Cashflow Handoff

Cashflow receives:

```text
actual bank balance
cleared cash
unreconciled items
expected receipts that failed to arrive
payments that failed
delayed settlements
fees/chargebacks
cash buffer status
```

Cashflow forecast updates.

Selene says:

```text
Customer DEF did not pay as expected. I’ve reduced the 7-day cash forecast and flagged collections.
```

This prevents cashflow from becoming a hopeful fiction.

## 23. Exception-Only Human Involvement

Selene auto-handles:

```text
exact receipt matches
exact payment matches
routine bank fees
known payment provider settlements
known internal transfers
low-value recurring bank charges
cleared supplier payments
cleared customer receipts
```

Humans review:

```text
unmatched large transactions
suspected fraud
unknown supplier payments
unknown customer receipts above threshold
payment mismatch
duplicate payment risk
unexpected bank fees
manual override
reopening locked reconciliation
cash transfer outside policy
```

Selene should not ask humans to approve every matched line.

## 24. PH1.D / GPT-5.5 Role

GPT-5.5 should be heavily used for explanation and messy-description understanding.

### GPT-5.5 May Help

```text
summarize reconciliation exceptions
interpret messy bank descriptions
draft bank query messages
draft supplier/customer explanation
explain cash position
explain why a transaction is unmatched
summarize monthly bank rec pack
prepare CFO briefing
translate bank/payment provider messages
```

### GPT-5.5 Must Not

```text
mark bank transaction as reconciled without deterministic proof
invent bank confirmation
approve suspicious transaction
post journal
move money
override reconciliation
alter bank account records
hide unmatched items
```

GPT-5.5 makes Selene sound human.

Deterministic Selene keeps the bank account safe.

## 25. Human-Like Selene Interaction

### Daily Cash

```text
Cleared operating cash is $82,200 after protecting payroll and tax reserves. Today’s supplier payments are safe to proceed.
```

### Bank Feed Issue

```text
The payroll account has not synced since yesterday. I’ll retry the connection and keep payroll cash marked as unverified until it updates.
```

### Unmatched Receipt

```text
I found a $3,000 receipt that likely belongs to Customer ABC, but the reference is missing. I need confirmation before matching it.
```

### Clean Reconciliation

```text
The operating account is reconciled for May. All bank lines are matched, and the close pack is ready.
```

### Suspicious Outflow

```text
This $8,500 bank transfer does not match any approved payment. I’ve marked it high-risk and opened an investigation.
```

No cold system gibberish. Selene speaks like a competent finance partner.

## 26. State Machines

### Bank Feed State

```text
NotConnected
PendingConnection
Connected
Syncing
Synced
SyncFailed
Disconnected
Suspended
Archived
```

### Bank Transaction State

```text
Imported
Normalized
Matched
ProbableMatch
Unmatched
Classified
AdjustmentProposed
Reconciled
ExceptionOpen
FraudReview
Archived
```

### Bank Reconciliation State

```text
NotStarted
InProgress
AutoMatched
ExceptionsOpen
ReadyForReview
Closed
Locked
Reopened
Archived
```

### Cash Position State

```text
Unknown
PartiallyVerified
Verified
Stale
HighRisk
ClosedForPeriod
```

## 27. Reason Codes

```text
BANK_FEED_CONNECTED
BANK_FEED_SYNCED
BANK_FEED_FAILED
BANK_STATEMENT_IMPORTED
BANK_TRANSACTION_IMPORTED
BANK_TRANSACTION_MATCHED
BANK_TRANSACTION_PROBABLE_MATCH
BANK_TRANSACTION_UNMATCHED
CUSTOMER_RECEIPT_MATCHED
SUPPLIER_PAYMENT_MATCHED
PAYROLL_PAYMENT_MATCHED
TAX_PAYMENT_MATCHED
POS_SETTLEMENT_MATCHED
ECOMMERCE_PAYOUT_MATCHED
BANK_FEE_DETECTED
INTEREST_DETECTED
FX_DIFFERENCE_DETECTED
CHARGEBACK_DETECTED
DUPLICATE_PAYMENT_RISK
UNKNOWN_OUTFLOW
UNKNOWN_RECEIPT
INTERNAL_TRANSFER_IN_TRANSIT
RECONCILIATION_CLOSED
RECONCILIATION_EXCEPTION_OPEN
CASH_POSITION_VERIFIED
CASH_POSITION_STALE
SUSPICIOUS_BANK_MOVEMENT
```

## 28. Required Simulations

```text
connect bank feed
import bank statement
auto-match customer receipt
auto-match supplier payment
auto-match POS settlement
auto-match e-commerce payout
detect bank fee
detect chargeback
detect FX difference
detect internal transfer
detect unknown supplier payment
detect unknown customer receipt
detect duplicate payment risk
payment provider settlement split
payroll batch cleared
supplier payment failed at bank
reconcile operating account
close bank reconciliation
reopen reconciliation
generate cash position
cashflow update from failed receipt
suspicious outflow investigation
```

## 29. Integration Map

```text
PH1.BANKREC / PH1.TREASURY
↔ PH1.ACCOUNTING
↔ PH1.FINANCE / CASHFLOW
↔ PH1.CREDITORS / AP
↔ PH1.SUPPLIER_PAYMENT
↔ PH1.AR / DEBTORS
↔ PH1.PAYROLL
↔ PH1.TAX
↔ PH1.POS
↔ PH1.ECOMMERCE
↔ PH1.B2B
↔ PH1.PAYMENT_PROVIDER
↔ PH1.BANKING
↔ PH1.ACCESS / AUTHORITY
↔ PH1.AUDIT
↔ PH1.WRITE
↔ PH1.D / GPT-5.5
```

## 30. Required Logical Packets

```text
BankAccountPacket
BankFeedConnectionPacket
BankStatementImportPacket
BankTransactionPacket
BankTransactionNormalizationPacket
BankMatchCandidatePacket
BankMatchDecisionPacket
ReceiptMatchPacket
PaymentMatchPacket
SettlementMatchPacket
BankFeePacket
FXDifferencePacket
ChargebackPacket
InternalTransferPacket
UnmatchedTransactionPacket
SuspiciousBankMovementPacket
BankReconciliationPacket
CashPositionPacket
TreasuryLiquidityPacket
AccountingBankHandoffPacket
CashflowBankHandoffPacket
AuditEvidencePacket
```

Logical only. Codex maps later. Do not create packet structs from this document alone.

## 31. What Codex Must Not Do

```text
Do not let BankRec own AP invoice truth.
Do not let BankRec own AR invoice truth.
Do not let BankRec calculate payroll.
Do not let BankRec decide tax treatment.
Do not let BankRec execute payments.
Do not let GPT-5.5 mark reconciliation closed.
Do not treat expected cash as cleared cash.
Do not treat payment submitted as payment confirmed.
Do not hide unmatched bank transactions.
Do not post ledger directly from bank feed without Accounting owner.
Do not implement from this document alone.
```

## 32. Final Architecture Sentence

Selene Bank Reconciliation + Treasury Control Engine is the accounting reality layer that imports and normalizes bank data, automatically matches receipts, supplier payments, payroll, tax, POS settlements, e-commerce payouts, payment provider reports, fees, interest, FX, chargebacks, and internal transfers, identifies unmatched or suspicious movements, verifies daily cash position, updates cashflow, supports treasury liquidity decisions, prepares accounting handoffs and reconciliation close packs, and uses GPT-5.5 to explain exceptions humanly while deterministic Selene proof controls reconciliation, matching, and financial truth.

Simple version:

```text
Selene imports the bank.
Selene matches what cleared.
Selene proves what was paid.
Selene proves what was received.
Selene finds what does not belong.
Selene updates cash position.
Selene prepares the reconciliation.
Selene tells Accounting what is real.
Humans only review exceptions.
```

The bank becomes part of Selene's truth loop, not an after-the-fact mystery.
<!-- END_SOURCE_FILE: docs/SELENE_FINANCE_ACCOUNTING_DOCUMENT_11_BANK_RECONCILIATION_TREASURY_CONTROL_MASTER_DESIGN.md -->

---

<!-- BEGIN_SOURCE_FILE: docs/SELENE_FINANCE_ACCOUNTING_DOCUMENT_12_CASHFLOW_WORKING_CAPITAL_OPTIMIZATION_MASTER_DESIGN.md -->
# Finance / Accounting Document 12 — Selene Cashflow Forecasting + Working Capital Optimization Engine

```text
DOCUMENT TYPE:
FINANCE / ACCOUNTING MASTER DESIGN

DOCUMENT NUMBER:
12

ENGINE:
PH1.CASHFLOW / PH1.WORKING_CAPITAL

FULL NAME:
Selene Cashflow Forecasting, Working Capital Optimization, Liquidity Protection, Receivables Acceleration, Sales Activation, and Payment Prioritization Engine

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

This document defines future canonical architecture for PH1.CASHFLOW / PH1.WORKING_CAPITAL. Repo-truth activation, simulation mapping, owner mapping, tests, and approved implementation slices must happen later before runtime behavior can be claimed.

The supplier early/urgent payment and Automation/Event Trigger Fabric addendum content is incorporated into this document as major sections. No separate addendum file is created in this batch.

## 1. Purpose

Selene Cashflow + Working Capital Engine owns the company's forward-looking liquidity brain.

It answers:

```text
How much cash do we really have?
How much cash will we have tomorrow, next week, next month, and next quarter?
Which customers are likely to pay late?
Which payments must be protected?
Which supplier payments can safely wait?
Which sales, collections, discounts, campaigns, or B2B actions can improve cash?
Which inventory purchases will hurt or help cash?
Will payroll, tax, rent, loans, or critical suppliers be covered?
When does cash become risky?
What should Selene do before humans start panicking?
```

Document 11 proves bank truth.

Document 12 uses that truth to forecast survival, opportunity, pressure, and action.

Profit is not cash.

Revenue is not cash.

Invoice issued is not cash.

"Customer promised Friday" is not cash.

Cash is cash.

## 2. Core Selene Law

```text
Selene must protect cash before crisis.

First collect what is owed.
Second increase incoming cash through sales, offers, B2B, marketing, and customer activation.
Third manage inventory and working capital intelligently.
Fourth prioritize outgoing payments.
Only then escalate to emergency finance decisions.
```

The order matters:

```text
1. Chase receivables.
2. Increase sales / bring cash forward.
3. Optimize inventory and working capital.
4. Prioritize outgoing payments.
5. Escalate cash shortfall.
```

A weak system says:

```text
Cash is tight. Delay bills.
```

Selene says:

```text
Cash will be tight in 14 days. I’m chasing $62,000 in likely collectible receivables, preparing a high-margin product campaign, delaying only flexible payments, and protecting payroll/tax/rent.
```

That is the standard.

## 3. Engine Boundary

### 3.1 PH1.CASHFLOW Owns

```text
cash position forecasting
cash inflow prediction
cash outflow prediction
working capital analysis
receivables acceleration recommendations
sales activation recommendations
payment prioritization recommendations
supplier payment timing recommendations
liquidity risk detection
cash buffer monitoring
cash shortfall alerts
13-week cashflow forecast
daily/weekly/monthly cash forecast
scenario cash modelling
cash confidence scoring
cashflow action plan
```

### 3.2 PH1.CASHFLOW Does Not Own

```text
bank transaction proof
ledger posting
invoice creation
supplier invoice validation
payment execution
customer credit terms ownership
tax law
payroll calculation
marketing campaign execution
product pricing authority
inventory stock truth
```

### 3.3 Correct Owner Split

```text
BankRec/Treasury = actual bank truth and reconciled cash.
Cashflow = forecast and liquidity action plan.
AR/Debtors = customer invoices, receipts, collections.
AP/Creditors = supplier invoices and payable amounts.
Supplier Payment = payment scheduling/execution handoff.
Accounting = ledger and financial statements.
Finance/Budget = strategic budget and spending policy.
Inventory = stock and reorder demand.
Product/Marketing/Sales = revenue activation levers.
Access/Authority = protected approvals.
Audit = proof.
```

Cashflow does not execute everything.

Cashflow tells Selene what must happen next to protect liquidity.

## 4. Cashflow Is Not Accounting Profit

Selene must clearly separate:

```text
profit
cash
revenue
receivables
payables
bank balance
available cash
restricted cash
forecast cash
```

A company can be profitable and still run out of cash.

Example:

```text
Revenue this month: $200,000
Cash collected: $60,000
Supplier bills due: $90,000
Payroll due: $40,000
Tax due: $20,000
```

Selene says:

```text
The company is profitable on paper, but cash collection is behind. Payroll and tax will create pressure unless receivables are collected or flexible payments are delayed.
```

This engine must stop humans from confusing sales with money in bank.

## 5. Cash Position Model

Selene calculates multiple cash layers.

```text
bank balance
cleared cash
pending payments
pending receipts
restricted cash
payroll reserve
tax reserve
loan covenant reserve
escrow/merchant clearing
available operating cash
forecast cash
risk-adjusted forecast cash
```

### 5.1 Basic Formula

```text
Opening cleared cash
+ expected customer receipts
+ expected POS/e-commerce settlements
+ expected B2B receipts
+ expected financing/investment inflows
- supplier payments
- payroll
- tax
- rent
- loan repayments
- subscriptions
- inventory purchases
- insurance
- capital expenditure
- other commitments
= forecast closing cash
```

### 5.2 Risk-Adjusted Forecast

Selene discounts uncertain inflows.

```text
Expected customer receipt × probability of payment
Expected sales × probability of conversion
Expected funding × probability of receipt
```

Example:

```text
Customer ABC owes $20,000.
Historically pays 12 days late.
Probability of payment this week: 35%.
Risk-adjusted cash inflow this week: $7,000.
```

Selene must not treat unreliable promises as full cash.

## 6. Forecast Horizons

Selene must forecast across multiple horizons.

```text
today
tomorrow
7 days
14 days
30 days
60 days
90 days
13-week rolling forecast
12-month rolling forecast
scenario forecast
```

### 6.1 Daily Cash Control

Used for:

```text
payment decisions
cash shortfall detection
supplier payment timing
payroll/tax protection
same-week collections
```

### 6.2 13-Week Cashflow

Used for:

```text
operational cash planning
working capital pressure
collections priority
inventory purchases
supplier payment strategy
management reporting
```

### 6.3 12-Month Rolling Forecast

Used for:

```text
strategic planning
seasonality
budget comparison
hiring plans
capital purchases
loan repayment planning
board reporting
```

Selene should update forecasts continuously.

## 7. Cashflow Data Sources

Selene uses data from:

```text
BankRec actual bank balances
AR invoices
AR aging
customer payment history
collections promises
customer credit scores
POS sales
e-commerce orders
B2B orders
subscription billing
supplier bills
AP aging
payment schedules
payroll schedules
tax obligations
rent/lease schedules
loan repayments
insurance premiums
inventory reorder forecasts
procurement commitments
asset purchases
marketing campaigns
sales pipeline
seasonality
public holidays/events
supplier terms
payment provider settlements
currency/FX exposure
```

Selene must tag data quality:

```text
Actual
Confirmed
Scheduled
Forecast
Estimated
Assumed
LowConfidence
```

Cashflow with no confidence score is not trusted.

## 8. Cashflow Confidence Score

Every forecast has a confidence score.

Inputs:

```text
reconciled bank freshness
AR payment reliability
customer promise reliability
AP certainty
payroll certainty
tax certainty
inventory forecast accuracy
sales forecast quality
seasonality confidence
supplier payment timing certainty
bank/payment provider settlement reliability
```

Score:

```text
HighConfidence
MediumConfidence
LowConfidence
Unreliable
```

Selene says:

```text
The 7-day forecast is high confidence because bank and AP data are current. The 60-day forecast is medium confidence because sales forecast assumptions are still being tested.
```

No false certainty.

## 9. Cashflow Risk Modes

Selene classifies company cash condition.

```text
GREEN — healthy
YELLOW — tightening
ORANGE — risk building
RED — cash shortfall likely
BLACK — critical intervention required
```

### GREEN

```text
pay by normal schedule
continue normal operations
monitor
```

### YELLOW

```text
increase collections reminders
watch discretionary spend
delay non-urgent purchases
monitor supplier commitments
```

### ORANGE

```text
activate collections protocol
activate sales/marketing cash campaign
review inventory reorders
prioritize payments
hold optional spend
notify finance owner
```

### RED

```text
protect payroll, tax, rent, critical suppliers
escalate to finance leadership
negotiate supplier terms
accelerate receivables
pause discretionary spend
run cash rescue plan
```

### BLACK

```text
emergency authority required
freeze non-essential payments
executive/board escalation if configured
legal/finance review if insolvency risk exists
```

Selene should never quietly drift into RED.

## 10. First Response — Receivables Acceleration

When cash is tightening, Selene first chases money owed.

She checks:

```text
overdue invoices
invoices due soon
large receivables
customers likely to pay late
broken promises
disputed invoices
customers with clean invoices
customers with payment links
customers with strong payment history
```

Selene ranks collection priority by:

```text
amount
days overdue
likelihood of collection
cashflow urgency
customer relationship value
dispute status
payment promise history
payment method availability
```

### 10.1 Autonomous Collections Actions

Selene can:

```text
send polite reminders
send payment links
send statements
ask for promise-to-pay
record promise-to-pay
chase broken promises
alert account manager
pause disputed portion only
escalate serious overdue accounts
suggest payment plans under policy
```

Selene says:

```text
We have a projected cash gap in 12 days. I’m chasing the top eight collectible invoices worth $54,000 before adjusting supplier payments.
```

Collect money owed before squeezing suppliers.

## 11. Second Response — Increase Incoming Sales

If receivables acceleration is not enough, Selene activates revenue levers.

Selene asks Product, Inventory, Marketing, E-Commerce, POS, and B2B:

```text
What can sell quickly?
What has stock available?
What has good margin?
What can be discounted safely?
What can be bundled?
What can be promoted to existing customers?
What B2B offers can bring fast cash?
Which customers are likely to reorder?
Which products create repeat buying habit?
```

### 11.1 Sales Activation Options

```text
flash sale
early payment discount
bundle
B2B bulk offer
repeat customer offer
subscription/renewal reminder
abandoned quote follow-up
slow-stock clearance
high-margin product push
preorder campaign
deposit campaign
loyalty activation
seasonal offer
```

### 11.2 Cash-Aware Marketing Rule

Marketing should not run random campaigns.

Cashflow tells Marketing:

```text
cash gap amount
needed collection window
target margin
stock availability
customer segment
discount limit
campaign urgency
```

Selene says:

```text
To close the $18,000 cash gap, I recommend a 72-hour B2B bundle campaign for overstocked items with strong margin. This is better than discounting high-demand stock.
```

Marketing becomes a cashflow lever, not noise.

## 12. Product Habit Logic

Selene must understand that some low-margin products create customer behavior.

Products may be:

```text
profit drivers
traffic drivers
habit builders
basket builders
loss leaders
strategic products
clearance products
dead stock
```

A low-margin product may still be valuable if it causes:

```text
repeat visits
higher basket value
cross-sell purchases
customer loyalty
B2B reorder behavior
subscription retention
```

Example:

```text
Milk margin is low.
Customers buying milk also buy bread, snacks, and higher-margin items.
Selene keeps milk stocked leanly and uses it as a habit product.
```

Selene says:

```text
This product has low margin, but it drives repeat customer visits and higher-margin basket purchases. I recommend keeping it but optimizing stock tightly.
```

This matters for cashflow because not all margin decisions are obvious at product-line level.

## 13. Third Response — Inventory Working Capital Optimization

Inventory ties up cash.

Selene must decide when to:

```text
buy less
buy later
split orders
transfer stock
discount overstock
pause reorder
switch supplier
use JIT
increase safety stock only where necessary
clear dead stock
```

Cashflow uses Inventory signals:

```text
stock value
days of cover
stockout risk
overstock risk
expiry risk
supplier lead time
sales velocity
reorder quantity
cash tied up
storage cost
```

Selene says:

```text
This reorder protects availability but ties up $42,000 in stock. Based on sales velocity and supplier lead time, I recommend ordering 60% now and reviewing again in 10 days.
```

For JIT:

```text
Cakes should be baked in two stages today: 40 before opening and 15 at 11 AM if sales pace confirms. Today’s public holiday increases demand, but staged baking reduces waste.
```

That is cashflow-aware inventory.

## 14. Fourth Response — Payment Management

Only after collection and sales actions does Selene optimize outgoing payments.

Payment classes:

```text
MustPay
CriticalSupplier
StrategicSupplier
NormalDue
Flexible
DisputedHold
Optional
DoNotPay
```

### 14.1 MustPay

```text
payroll
tax
rent
loan repayment
insurance critical to operation
legal/statutory obligation
```

### 14.2 CriticalSupplier

```text
supplier required for production
supplier required for customer delivery
supplier needed for operations
supplier where delay causes business harm
```

### 14.3 Flexible

```text
non-critical supplier
within terms
no discount benefit
no penalty
low relationship impact
```

### 14.4 DoNotPay

```text
duplicate invoice
supplier bank risk
unmatched invoice
disputed goods
missing credit note
supplier blocked
```

Selene says:

```text
Pay payroll, tax, rent, and the critical packaging supplier. Hold disputed invoices. Schedule flexible suppliers on their due dates.
```

Humans do not need to approve the obvious under policy. They approve exceptions.

## 15. Supplier Early / Urgent Payment Requests

When suppliers request early or urgent payment, or offer early-payment discounts, Selene must evaluate invoice readiness, supplier risk, bank safety, cashflow, discount benefit, strategic importance, and company policy before deciding whether to pay early, pay urgently, negotiate, schedule normally, hold, or escalate. Routine approved requests may be handled autonomously. Exceptions require authority.

### 15.1 Supplier Asks For Early Payment

When a supplier asks:

```text
Can you pay us early?
```

Selene checks:

```text
invoice is valid
invoice is matched
goods/services were accepted
no credit note is pending
no supplier dispute exists
supplier bank is verified
supplier is approved
supplier is not restricted
cashflow can support early payment
early payment has a business reason
```

Then Selene decides:

```text
approve automatically under policy
recommend approval
offer partial early payment
ask supplier for discount
reject politely
escalate only if outside policy
```

Reasons may include:

```text
supplier is critical
supplier relationship is strategic
supplier gives discount
early payment secures supply
late supply risk exists
company has surplus cash
```

Selene says:

```text
This invoice is clean and payable, but it is not due for 18 days. Cashflow is healthy, so I can approve early payment only if it benefits the company or fits policy.
```

Selene should not pay early for free unless there is a reason.

### 15.2 Supplier Asks For Urgent Payment

Urgent payment is treated differently.

Supplier says:

```text
We need urgent payment today.
```

Selene checks:

```text
why urgent?
is supplier critical?
will supply stop?
will customer delivery be affected?
is invoice clean?
is payment already due?
is cashflow safe?
is bank verified?
is this supplier abusing urgent requests?
```

Selene can respond:

```text
I can see the invoice is valid, but it is not due yet. Please confirm the reason for urgent payment.
```

Then Selene classifies:

```text
valid urgency
supplier pressure only
cashflow negotiation
supply continuity risk
fraud/risk signal
```

If valid and under policy, Selene can act.

If not, Selene says:

```text
This payment is not due yet and no approved urgency reason is recorded. I can schedule it for the normal due date or request an early-payment discount.
```

"Urgent for supplier" does not automatically mean "urgent for us."

### 15.3 Supplier Offers Early-Payment Discount

Supplier says:

```text
Pay today and we’ll give 2% discount.
```

Selene calculates:

```text
discount amount
days paid early
annualized benefit
cashflow impact
supplier importance
upcoming payroll/tax/rent
available cash buffer
alternative use of cash
risk of paying before due date
```

Decision rule:

```text
Take discount if:
discount benefit > cost/risk of using cash
AND cash buffer remains safe
AND invoice is clean
AND supplier bank is verified
```

Example:

```text
Invoice: $10,000
Discount: 2% = $200
Due date: 20 days away
Cashflow: GREEN
Result: take discount
```

Selene says:

```text
The supplier is offering a $200 discount for early payment. Cashflow remains above buffer, so I recommend taking it.
```

If cashflow is tight:

```text
The discount is attractive, but paying early would reduce the payroll buffer. I recommend declining the discount and paying on the due date.
```

Selene can also negotiate:

```text
We can pay early if the supplier increases the discount to 3% or allows partial payment.
```

### 15.4 Supplier Payment Request Flow

```text
Supplier request received
→ Selene identifies supplier
→ Checks invoice/payment readiness
→ Checks disputes/credits
→ Checks supplier bank safety
→ Checks cashflow
→ Checks payment policy
→ Calculates benefit/risk
→ Auto-acts if policy allows
→ Escalates only if exception
→ Sends supplier response
→ Audits everything
```

Possible outcomes:

```text
pay early
pay urgently
pay partially
pay on due date
ask for discount
ask for reason
reject request
hold due to dispute
hold due to bank risk
escalate to finance
```

## 16. Working Capital Metrics

Selene tracks:

```text
cash conversion cycle
days sales outstanding
days payables outstanding
days inventory outstanding
working capital ratio
current ratio
quick ratio
cash buffer days
inventory cash tied up
receivables at risk
payables due pressure
```

Selene explains in human language:

```text
Cash is tightening because customers are paying slower, inventory purchases increased, and supplier bills are due before the receivables are expected.
```

Internal metrics like DSO, DPO, DIO, and cash conversion cycle are useful internally. Human-facing output should explain what they mean.

## 17. Forecast Scenarios

Selene supports scenarios:

```text
Base
Optimistic
Pessimistic
Cash Rescue
High Growth
Supplier Delay
Customer Late Payment
Sales Campaign Success
Inventory Purchase Delay
Payroll Expansion
Tax Shock
```

Scenario inputs:

```text
sales increase/decrease
collection delay
supplier payment delay
inventory order size
payroll increase
tax payment
loan payment
marketing campaign effect
customer default risk
```

Selene says:

```text
If the largest customer pays 14 days late, cash drops into RED on 18 June. If we collect two mid-size invoices and delay flexible supplier payments within terms, cash remains YELLOW.
```

## 18. Autonomous Cashflow Actions

Selene can automatically:

```text
update daily cash forecast
detect cash tightening
rank receivables to chase
send payment reminders
send payment links
record payment promises
detect broken promises
trigger collections workflow
recommend sales/marketing campaign
recommend B2B offer
recommend early payment discount to customers
recommend inventory reorder split
pause non-critical purchase recommendation
prioritize supplier payments
schedule flexible payments within terms
protect payroll/tax/rent buffers
prepare cashflow report
prepare management alert
prepare board cash pack if required
```

Selene needs authority for:

```text
delaying protected supplier payments
changing credit terms
offering major discounts
settlement/payment plans outside policy
drawing finance/loan
moving large funds between accounts
freezing departments
emergency cash actions
board-level cash decisions
```

Routine action under approved policy should not ask humans.

Exception action should.

## 19. Selene Autonomous Trigger + Event Fabric

Autonomous Selene work is driven by triggers, not humans remembering chores.

Each engine owns its business triggers, but all triggers must flow through a central Automation/Event Fabric so Selene can coordinate timing, risk, policy, simulation, authority, action, communication, and audit across engines.

Future central fabric:

```text
PH1.AUTOMATION / PH1.EVENT
Selene Autonomous Trigger + Event Fabric
```

Each engine registers:

```text
what events it listens for
what schedule it runs on
what thresholds matter
what actions it may auto-run
what requires authority
what evidence it must produce
```

### 19.1 Trigger Types

Scheduled triggers run by time.

Examples:

```text
daily bank feed check
daily cash position
morning receiving manifest
weekly supplier review
month-end AP close
quarterly tax reminder
```

Event triggers run when something happens.

Examples:

```text
invoice received
payment confirmed
bank transaction imported
goods marked damaged
credit note received
customer payment promise broken
supplier changed bank details
stock drops below reorder point
```

Threshold triggers run when a number crosses a limit.

Examples:

```text
cash buffer below target
stock below safety level
customer over credit limit
supplier quality score drops
AP overdue above threshold
inventory expiry risk high
```

Risk triggers run when risk changes.

Examples:

```text
supplier bank changed
cashflow moves to ORANGE
supplier becomes restricted
large unmatched bank transaction appears
duplicate invoice detected
```

Human voice/text triggers run from human requests.

Examples:

```text
"Selene, pay this supplier."
"Selene, reorder stock."
"Selene, show today’s cash."
"Selene, why is this invoice held?"
```

Selene-to-Selene triggers run between companies using Selene.

Examples:

```text
supplier sends credit note
supplier asks early payment
supplier changes bank details
supplier confirms shipment delay
buyer sends receiving variance
```

### 19.2 Trigger Pipeline

Every trigger follows this pipeline:

```text
Trigger fires
→ PH1.N understands meaning
→ PH1.X classifies risk
→ Source engine validates truth
→ Policy checks if Selene can act
→ Simulation runs
→ Access/Authority checks if needed
→ Selene acts or escalates
→ PH1.WRITE explains result
→ Audit records proof
```

A trigger does not blindly execute.

It becomes one of:

```text
auto-action
recommendation
reminder
exception
blocked action
authority request
```

### 19.3 Engine Trigger Examples

BankRec triggers:

```text
bank feed sync
payment batch submitted
supplier payment confirmation expected
customer receipt expected
daily cash position schedule
month-end close
```

Selene action:

```text
match bank lines
detect unmatched items
update cash position
update cashflow forecast
```

Cashflow triggers:

```text
bank balance update
invoice issued
customer receipt received
payment promise broken
AP invoice ready
supplier asks urgent payment
supplier asks early payment
payroll due soon
inventory reorder proposed
sales forecast changes
```

Selene action:

```text
reforecast cash
prioritize collections
recommend sales campaign
adjust supplier payment schedule
protect payroll/tax/rent
```

Supplier Payment triggers:

```text
AP marks invoice ready
supplier asks early payment
supplier offers discount
payment due date approaching
cashflow turns GREEN/YELLOW/ORANGE
supplier bank risk changes
```

Selene action:

```text
schedule payment
take discount if safe
hold payment if risk
send remittance
escalate exception
```

Receiving triggers:

```text
today’s expected deliveries
delivery arrives
delivery does not arrive
receiver uploads photo
damage detected
inspection completed
```

Selene action:

```text
update manifest
record received quantity
request credit note
hold AP disputed amount
update supplier score
```

Inventory triggers:

```text
sale
B2B order
stock movement
supplier delay
public holiday
expiry approaching
forecast demand changes
```

Selene action:

```text
calculate reorder
recommend transfer
create draft PO
recommend discount
use JIT logic
```

### 19.4 Trigger Cadence Examples

```text
Bank feeds:
on sync + daily + before payment run

Cashflow:
after every major cash event + daily forecast + weekly management view

Inventory:
event-driven for sales/receipts + hourly/daily for fast-moving/perishable goods

Receiving:
daily morning manifest + delivery arrival + end-of-day closeout

AP:
on invoice receipt + on credit note + on receiving completion + before payment run

Supplier payments:
daily payment run + due-date trigger + supplier request trigger + discount trigger

Supplier statements:
on statement receipt + month-end + supplier dispute
```

The cadence changes by risk.

```text
High-risk / high-volume / perishable = frequent
Low-risk / slow-moving = less frequent
Cash crisis = much more frequent
```

## 20. PH1.D / GPT-5.5 Role

GPT-5.5 should be heavily used for cashflow explanation, drafting, and scenario narration.

### GPT-5.5 May Help

```text
explain cashflow risk in plain English
draft customer payment reminders
draft supplier delay/negotiation messages
draft management cash summary
summarize cashflow scenarios
explain why a payment is prioritized
prepare board cashflow narrative
prepare marketing campaign wording
summarize collections strategy
```

Example:

```text
Cash pressure is mainly caused by slow customer receipts and a large inventory reorder. Payroll and tax are protected, but supplier payments need careful scheduling.
```

### GPT-5.5 Must Not

```text
decide official cash balance
approve payment delay
change supplier terms
change customer terms
execute bank transfer
alter forecast data
hide cash risk
invent customer payments
invent sales projections
override finance policy
```

GPT-5.5 helps Selene talk like a capable finance partner.

Deterministic Selene keeps the numbers honest.

## 21. Human-Like Selene Interaction

### Early Warning

```text
Cash will tighten in about 12 days unless customer receipts arrive as expected. I’m chasing the highest-probability invoices first.
```

### Sales Activation

```text
Collections alone may not close the gap. I recommend a 72-hour offer on high-margin stock that is already available.
```

### Payment Prioritization

```text
Payroll, tax, rent, and critical suppliers are protected. I’ll schedule flexible suppliers on their due dates and hold disputed invoices.
```

### Inventory Cash Pressure

```text
This reorder is useful but too heavy for current cashflow. I recommend splitting it into two smaller orders.
```

### Management Summary

```text
The next 30 days are manageable if two overdue customers pay this week. If they miss payment, we enter ORANGE risk and should activate payment management.
```

Human-like, calm, clear, not internal exception codes.

## 22. Cashflow State Machines

### Cash Forecast State

```text
Draft
DataCollecting
ForecastGenerated
ConfidenceScored
ScenarioRunning
ActionPlanGenerated
Monitoring
Updated
Archived
```

### Cash Risk State

```text
Green
Yellow
Orange
Red
Black
Resolved
```

### Cash Action State

```text
Suggested
AutoActionedUnderPolicy
PendingAuthority
InProgress
Completed
Failed
Escalated
Archived
```

### Receivable Acceleration State

```text
Candidate
ReminderSent
PaymentLinkSent
PromiseRequested
PromiseReceived
Paid
BrokenPromise
Escalated
Closed
```

### Payment Prioritization State

```text
Unclassified
MustPay
CriticalSupplier
NormalDue
Flexible
DisputedHold
DoNotPay
Scheduled
Paid
```

## 23. Reason Codes

```text
CASH_FORECAST_UPDATED
CASH_POSITION_STALE
CASH_RISK_GREEN
CASH_RISK_YELLOW
CASH_RISK_ORANGE
CASH_RISK_RED
CASH_RISK_BLACK
RECEIVABLES_ACCELERATION_REQUIRED
CUSTOMER_PAYMENT_LATE_RISK
BROKEN_PAYMENT_PROMISE
SALES_ACTIVATION_RECOMMENDED
B2B_CASH_CAMPAIGN_RECOMMENDED
INVENTORY_REORDER_CASH_WARNING
PAYMENT_PRIORITY_MUST_PAY
PAYMENT_PRIORITY_FLEXIBLE
PAYMENT_DELAY_WITHIN_TERMS_RECOMMENDED
PAYMENT_BLOCKED_DISPUTE
PAYROLL_BUFFER_PROTECTED
TAX_BUFFER_PROTECTED
CASHFLOW_ACTION_PLAN_CREATED
BOARD_CASH_ESCALATION_REQUIRED
SUPPLIER_EARLY_PAYMENT_REQUESTED
SUPPLIER_URGENT_PAYMENT_REQUESTED
EARLY_PAYMENT_DISCOUNT_OFFERED
AUTOMATION_TRIGGER_FIRED
AUTOMATION_TRIGGER_BLOCKED_AUTHORITY
```

## 24. Required Simulations

```text
generate 7-day cash forecast
generate 13-week cash forecast
cash gap detected
receivables chase activated
customer payment promise risk-adjusted
broken promise updates forecast
sales campaign recommended for cash gap
B2B offer recommended
inventory reorder split for cash protection
supplier payment prioritization
payroll/tax/rent protected
cashflow blocks early supplier payment
cashflow approves early payment discount
supplier asks early payment
supplier asks urgent payment
supplier offers early-payment discount
supplier urgent request rejected without valid reason
automation scheduled trigger fires
automation event trigger fires
automation threshold trigger fires
automation risk trigger fires
automation human trigger fires
automation Selene-to-Selene trigger fires
cashflow scenario: largest customer late
cashflow scenario: public holiday sales uplift
cashflow scenario: inventory purchase delay
cash risk moves Green to Yellow
cash risk moves Orange to Red
board escalation for Red/Black cash risk
management cashflow summary generated
```

## 25. Integration Map

```text
PH1.CASHFLOW / PH1.WORKING_CAPITAL
↔ PH1.BANKREC / TREASURY
↔ PH1.ACCOUNTING
↔ PH1.FINANCE / BUDGET
↔ PH1.AR / DEBTORS
↔ PH1.AR.COLLECT
↔ PH1.CREDITORS / AP
↔ PH1.SUPPLIER_PAYMENT
↔ PH1.PAYROLL
↔ PH1.TAX
↔ PH1.INVENTORY
↔ PH1.PRODUCT
↔ PH1.PROCUREMENT
↔ PH1.ECOMMERCE
↔ PH1.B2B
↔ PH1.POS
↔ PH1.MARKETING
↔ PH1.CUSTOMER
↔ PH1.BOARD
↔ PH1.ACCESS / AUTHORITY
↔ PH1.AUDIT
↔ PH1.WRITE
↔ PH1.D / GPT-5.5
↔ PH1.AUTOMATION / PH1.EVENT
↔ PH1.N
↔ PH1.X
```

## 26. Required Logical Packets

```text
CashPositionPacket
CashForecastPacket
CashForecastConfidencePacket
CashRiskPacket
CashActionPlanPacket
ReceivablesAccelerationPacket
SalesActivationCashPacket
InventoryCashImpactPacket
PaymentPriorityPacket
SupplierEarlyPaymentRequestPacket
SupplierUrgentPaymentRequestPacket
EarlyPaymentDiscountPacket
SupplierPaymentRequestDecisionPacket
AutomationTriggerPacket
ScheduledTriggerPacket
EventTriggerPacket
ThresholdTriggerPacket
RiskTriggerPacket
HumanTriggerPacket
SeleneToSeleneTriggerPacket
TriggerDecisionPacket
WorkingCapitalMetricsPacket
CashScenarioPacket
CashflowManagementAlertPacket
CashflowBoardEscalationPacket
CashflowAuditEvidencePacket
```

Logical only. Codex maps later. Do not create packet structs from this document alone.

## 27. What Codex Must Not Do

```text
Do not confuse profit with cash.
Do not treat expected receipts as cleared cash.
Do not let Cashflow execute payments.
Do not let Cashflow post accounting.
Do not let GPT-5.5 invent forecast numbers.
Do not hide cash shortfall risk.
Do not delay protected payments without authority.
Do not start payment management before receivables/sales actions.
Do not ignore inventory cash impact.
Do not require human approval for routine cashflow actions under policy.
Do not pay suppliers early without invoice readiness, bank safety, cashflow, policy, and benefit/risk checks.
Do not treat supplier urgency as company urgency without evidence.
Do not take early-payment discounts if cash buffers become unsafe.
Do not let triggers blindly execute.
Do not bypass PH1.N, PH1.X, source owners, simulation, Access/Authority, PH1.WRITE, or Audit in trigger handling.
Do not implement from this document alone.
```

## 28. Final Architecture Sentence

Selene Cashflow Forecasting + Working Capital Optimization Engine is the forward-looking liquidity brain that uses reconciled bank truth, receivables, payables, payroll, tax, inventory, sales, B2B, POS, e-commerce, supplier terms, customer payment behavior, and cash buffers to forecast cash, detect risk early, first accelerate receivables, then activate sales and marketing cash levers, then optimize inventory and outgoing payments, protect critical obligations, evaluate supplier early/urgent payment requests and early-payment discounts, coordinate autonomous triggers through the Automation/Event Fabric, generate scenarios, and explain everything humanly through GPT-5.5 while deterministic Selene engines keep numbers, authority, execution, and audit safe.

Simple version:

```text
Selene knows real cash.
Selene predicts future cash.
Selene chases money owed first.
Selene increases sales second.
Selene manages payments third.
Selene protects payroll, tax, rent, and critical suppliers.
Selene optimizes inventory cash.
Selene checks supplier early or urgent payment requests before acting.
Selene uses triggers so work happens on time.
Selene warns before crisis.
Humans approve only exceptions.
Everything is audited.
```

Cashflow is not merely a report. It is Selene's survival brain.
<!-- END_SOURCE_FILE: docs/SELENE_FINANCE_ACCOUNTING_DOCUMENT_12_CASHFLOW_WORKING_CAPITAL_OPTIMIZATION_MASTER_DESIGN.md -->
