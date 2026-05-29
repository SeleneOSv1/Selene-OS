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
