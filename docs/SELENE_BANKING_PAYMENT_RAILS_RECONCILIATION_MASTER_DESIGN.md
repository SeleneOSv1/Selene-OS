# Selene Banking + Payment Rails + Reconciliation Master Design

```text
DOCUMENT TYPE:
MASTER DESIGN / BANKING + PAYMENT RAILS + BANK FEEDS + PAYMENT EXECUTION + RECONCILIATION

STATUS:
MASTER_DESIGN
NOT_RUNTIME_IMPLEMENTATION
PENDING_GRAND_ARCHITECTURE_RECONCILIATION

PURPOSE:
Define Selene's Banking + Payment Rails + Reconciliation system: bank connections, payment instructions, bank API/payment-provider execution, payment approvals, bank confirmations, direct bank receipts, payment matching, bank reconciliation, payment failures, bank/card fees, chargebacks, payment reversals, FX/payment rails, cash visibility, and accounting handoff.
```

## 0. Authority and Scope

AGENTS.md controls execution.

This is a docs-only master design.

No runtime code was changed.

This document does not authorize implementation.

Current repo truth does not prove a complete runtime Banking, Payment Provider, Payment Rails, or Reconciliation owner. This document defines future architecture pending Grand Architecture Reconciliation.

Future implementation requires repo-truth activation, approved file scope, tests, backend evidence, provider-off/fake-provider proof, Access/Authority proof, audit proof, and JD approval.

## 1. Executive Target

Banking is Selene's proof layer for money movement.

Accounts Payable says:

```text
We owe money.
```

Accounts Receivable says:

```text
They owe us money.
```

Payroll says:

```text
Employees must be paid.
```

Accounting says:

```text
The books need the correct journal.
```

Finance/Cashflow says:

```text
Can we afford this and should we prioritize it?
```

Banking says:

```text
Money actually moved, failed, reversed, settled, cleared, or remains unmatched.
```

The target:

```text
Selene connects to banks and payment providers.
Selene receives bank feeds automatically.
Selene prepares payment instructions from approved source owners.
Selene sends payments only through protected authority gates.
Selene confirms when the bank/provider executes.
Selene reconciles payments and receipts automatically.
Selene identifies who paid.
Selene detects ambiguous payments.
Selene handles failed, partial, reversed, or chargeback payments.
Selene checks abnormal bank/card/payment fees.
Selene never guesses money truth.
```

Tiny translation: Banking is where Selene stops talking about money and proves where it went. Which is useful, because banks are basically truth machines with fees and bad user interfaces.

## 2. Master Law

```text
No payment execution without approved source instruction.

No bank API call without Access, Authority, Simulation, provider proof, and audit.

No payment marked paid until bank/payment provider confirmation exists.

No receipt marked allocated until AR/Banking matching proves the customer/invoice.

No supplier/employee/customer bank detail change without protected verification.

No reconciliation by guessing.

No raw bank credentials in logs.

No GPT-5.5 payment execution.

No client, adapter, or UI owns payment truth.
```

## 3. Banking Owner Split

### Banking / Payment Rails Owns

```text
bank connections
bank account feeds
bank balances
payment provider connections
payment instruction intake
payment execution handoff
payment status tracking
bank/payment confirmation
payment failure proof
bank transaction evidence
payment rail cutoff data
settlement timing
bank reconciliation
payment provider reconciliation
direct deposit detection
bank/card fee evidence
chargeback/reversal evidence
```

### Accounts Payable Owns

```text
supplier bill readiness
contractor invoice readiness
AP payment approval
payment due date
payment amount
supplier/payee obligation
AP payment schedule
```

AP does not execute bank transfer.

### Accounts Receivable Owns

```text
customer invoice balance
receipt allocation
customer payment status
unmatched receipt review
payment plan status
AR aging
```

AR does not own bank feed truth.

### Payroll Owns

```text
employee pay calculation
payrun approval
employee pay instruction
payslip
payroll payment readiness
payroll liabilities
```

Payroll does not execute bank transfer alone.

### Finance / Cashflow Owns

```text
cash availability
minimum cash reserve
payment priority
cashflow forecast
emergency reserve override
payment timing recommendation
bank account selection policy
```

### Accounting Owns

```text
bank journal posting
payment journal posting
receipt journal posting
bank fee journal posting
chargeback/reversal journal posting
reconciliation accounting entries
ledger truth
```

### Access / Governance Owns

```text
who can connect bank
who can view bank account
who can view balances
who can approve payment
who can send payment
who can approve refund
who can approve bank-detail changes
who can export bank data
who can override reserve
who can reconcile manually
```

### Tax / Compliance Owns

```text
tax remittance obligations
withholding payment obligations
GST/VAT/tax payment rules
country payment/reporting compliance
```

### PH1.BCAST / PH1.DELIVERY Owns

```text
payment approval request delivery
payment confirmation delivery
remittance advice delivery
failed payment notification
customer payment link delivery
supplier payment notice delivery
```

### PH1.REM Owns

```text
payment approval follow-up timing
bank failure follow-up timing
reconciliation reminder timing
critical payment cutoff reminders
customer payment reminder timing
```

### PH1.WRITE Owns

```text
human payment explanations
bank failure wording
approval request wording
reconciliation explanations
customer payment received wording
supplier remittance wording
cash reserve warning wording
```

### PH1.D / GPT-5.5 May Assist

```text
explain bank reconciliation differences
draft payment approval summaries
suggest likely bank receipt matches
explain failed payment in simple language
summarize cash movement
draft customer/supplier payment messages
classify bank transaction descriptions as candidates
```

But must not:

```text
execute payment
approve payment
mark invoice paid
mark bill paid
change bank details
decide final reconciliation
invent bank confirmation
override cash reserve
post journal
```

## 4. Banking Scope

Selene Banking must support:

```text
bank account connection
bank balance reading
bank transaction feeds
bank payment instructions
bank transfer execution through provider gates
payment provider status
payment confirmation
direct bank deposits
bank reconciliation
credit card settlement evidence
payment provider fees
bank fees
chargebacks
refund payment proof
failed payments
partial payment failures
bank account validation where provider exists
payment file export where API unavailable
open banking where available
manual bank statement import where required
multi-currency payment evidence
FX settlement evidence
remittance advice
```

Banking must not own:

```text
supplier bill truth
customer invoice truth
payroll calculation truth
ledger posting truth
tax law truth
budget policy
access permission
delivery wording
```

## 5. Bank Account Setup

A company may have multiple bank accounts.

### Bank Account Fields

```text
bank_account_id
tenant_id
company_id
legal_entity_id
country
region
bank_name
bank_branch
account_name
account_number_ref
routing_number_ref
bsb_ref
iban_ref
swift_bic_ref
currency
account_type
status
bank_provider_ref
feed_enabled
payment_enabled
reconciliation_enabled
last_feed_sync_at
created_by
approved_by
audit_ref
```

### Bank Account Statuses

```text
Draft
PendingVerification
ActiveFeedOnly
ActivePaymentEnabled
Suspended
PaymentHold
Closed
Archived
```

### Account Types

```text
operating_account
payroll_account
tax_account
trust_account
loan_account
savings_account
merchant_clearing_account
credit_card_account
foreign_currency_account
```

Bank credentials and account numbers must be protected. Audit logs use refs and last-4 style display only.

## 6. Bank Connection and Provider Proof

Selene may connect to banks through:

```text
bank API
open banking provider
payment provider
bank file upload/export
bank statement import
merchant settlement provider
manual bank confirmation with authority
```

### Bank Provider Connection Packet

```text
BankProviderConnectionPacket:
  connection_id
  company_id
  legal_entity_id
  bank_account_id
  provider_name
  provider_type
  capabilities:
    - balance_read
    - transaction_feed
    - payment_send
    - payment_status
    - bank_account_validation
    - reconciliation_feed
  consent_ref
  credential_ref
  connection_status
  last_verified_at
  audit_ref
```

### Connection Statuses

```text
NotConnected
PendingConsent
Connected
FeedActive
PaymentActive
ExpiredConsent
ProviderError
Revoked
Suspended
```

Selene must never store raw credentials in ordinary logs.

## 7. Payment Rails

Payment rails vary by country.

Selene must support future mapping for:

```text
domestic bank transfer
instant payment
ACH
SEPA
SWIFT
BACS
FPS
EFT
wire transfer
card refund
direct debit
BPAY where applicable
PayNow/FAST where applicable
UPI where applicable
check/cheque where legacy required
payment file export
provider wallet
merchant settlement
```

Payment rail fields:

```text
payment_rail_id
country
currency
rail_name
cutoff_time
processing_days
receiver_clearing_days
weekend_processing
public_holiday_processing
max_amount
requires_dual_approval
supports_confirmation
supports_reversal
supports_partial_failure
provider_ref
effective_from
effective_to
```

Critical rule:

```text
Due date is not send date.

Selene must calculate latest safe send/start date based on payment rail timing, approval lead time, public holidays, and clearing time.
```

This connects back to Document 3 Addendum.

## 8. Payment Instruction Intake

Banking does not invent payments.

It receives payment instructions from source owners.

Sources:

```text
Accounts Payable
Payroll
Refunds / AR
Tax / Compliance
Loan / Lease / Debt
Dividends / Equity
Expense Reimbursements
Intercompany Transfers
Treasury
```

### Payment Instruction Packet

```text
PaymentInstructionPacket:
  payment_instruction_id
  source_owner
  source_document_ref
  payment_type
  payer_company_id
  payer_legal_entity_id
  payer_bank_account_id
  payee_id
  payee_type
  payee_bank_ref
  amount
  currency
  payment_rail
  requested_payment_date
  due_date
  latest_safe_start_date
  approval_refs
  cashflow_check_ref
  simulation_ref
  step_up_refs
  payment_description
  remittance_ref
  status
  audit_ref
```

Payment instruction statuses:

```text
Draft
PendingSourceApproval
PendingCashflowCheck
PendingAuthority
PendingStepUp
ReadyForBank
SentToBank
BankAccepted
BankRejected
Processing
Confirmed
PartiallyFailed
Failed
Cancelled
Reversed
Reconciled
Archived
```

## 9. Payment Approval and Multi-Authority Control

Payment approval may require one or more approvers.

Examples:

```text
supplier payment over 100,000 requires CEO + CFO
refund over 10,000 requires Finance Manager + CFO
payroll commit requires Payroll Officer + CFO
bank transfer to new supplier requires Finance + Director
cash reserve breach requires CFO + CEO or board
```

### Approval Requirements

```text
action_type
amount_threshold
currency
legal_entity
payment_type
payee_risk
new_bank_account
budget_status
cash_reserve_status
required_approver_roles
number_of_approvers
step_up_required
expires_at
audit_ref
```

### Step-Up

Approvers may need:

```text
Face ID
fingerprint
secure passcode
approved device confirmation
voice confirmation only as evidence, not authority
```

Important law:

```text
Biometric/passcode proves approver presence.

It does not replace authority.

Authority comes from Access/Governance.
```

Example:

```text
Selene prepares payment.
Harry CEO approves with Face ID.
Bob CFO approves with Face ID.
Access confirms both authority scopes.
Only then Banking sends payment through bank API.
```

## 10. Payment Execution Boundary

Banking may execute only after all gates pass.

Required gates:

```text
valid payment instruction
source owner approval
Access authority
required approvers complete
step-up complete where required
cashflow check passed or emergency override approved
bank/payee details valid
payment rail selected
simulation passed
audit ready
provider available
```

Execution flow:

```text
source owner creates payment instruction
-> Finance/Cashflow checks payment impact
-> Access resolves authority
-> approvers approve with step-up if required
-> Banking submits to bank/payment provider
-> provider returns accepted/rejected/processing
-> Banking tracks until confirmed/failed
-> Accounting receives payment evidence
-> source owner updates status
-> reconciliation completes
```

Hard rule:

```text
No bank API call from AP, AR, Payroll, Finance, PH1.D, client, or Adapter directly.

Only Banking/Payment owner may send to payment provider after gates pass.
```

## 11. Payment Confirmation

A payment is not paid just because Selene sent it.

Payment confirmation requires provider/bank proof.

### Payment Confirmation Packet

```text
PaymentConfirmationPacket:
  confirmation_id
  payment_instruction_id
  provider_ref
  bank_transaction_id
  payment_status
  amount_sent
  amount_confirmed
  currency
  confirmed_at
  settlement_date
  payee_account_last4
  failure_reason
  fee_amount
  exchange_rate_ref
  audit_ref
```

Payment statuses:

```text
Submitted
Accepted
Processing
Confirmed
Rejected
Failed
PartiallyFailed
Returned
Reversed
Cancelled
Settled
Reconciled
```

Source owners update only after Banking confirms.

## 12. Payment Failures

Selene must handle payment failures cleanly.

Failure types:

```text
bank_rejected
provider_error
insufficient_funds
invalid_bank_account
account_closed
payee_name_mismatch
payment_limit_exceeded
duplicate_payment_risk
bank_cutoff_missed
public_holiday_delay
partial_file_failure
network_timeout
sanctions_or_compliance_hold
fraud_hold
returned_payment
```

Failure response:

```text
record provider proof
do not mark paid
notify source owner
notify approver if required
create payment exception
schedule retry if allowed
ask for corrected bank details if needed
escalate critical payment risk if due date threatened
audit
```

Selene says:

```text
The bank rejected this payment because the account details appear invalid. I've kept the bill unpaid and opened a payment exception for review.
```

## 13. Direct Bank Receipts

Customers may pay directly into bank accounts.

Banking receives bank feed evidence.

AR performs allocation.

### Bank Receipt Fields

```text
bank_transaction_id
bank_account_id
received_at
value_date
amount
currency
payer_name
payer_bank_ref
reference_text
description
payment_rail
provider_ref
raw_bank_feed_ref
match_status
audit_ref
```

Banking owns receipt evidence.

AR owns customer/invoice allocation.

Accounting owns journal posting.

## 14. Payment Matching / Cash Application

Selene must determine who paid and what invoice they paid.

Matching evidence:

```text
invoice number
customer name
payer bank account
amount
payment date
payment reference
payment link id
remittance advice
customer history
open invoice amounts
payment plan schedule
Selene-to-Selene payment confirmation
bank transaction description
```

### Match Confidence

```text
high_confidence
medium_confidence
low_confidence
ambiguous
no_match
```

### Matching Rule

```text
High-confidence matches may be auto-allocated if company policy allows.

Medium-confidence matches require review or customer confirmation.

Ambiguous matches must not be auto-allocated.

No-match receipts go to unmatched receipt queue.
```

Example:

```text
Two customers both owe $5,000.
Bank receipt says "Payment."
Selene must not guess.
Selene creates unmatched receipt review and asks for remittance/proof.
```

## 15. Customer Says "I Paid"

This is a special Selene capability.

Flow:

```text
customer claims paid
-> Selene asks for payment date/amount/reference if needed
-> Banking searches bank feeds/provider records
-> AR searches unmatched receipts
-> candidate matches ranked
-> if high confidence, allocate receipt
-> if ambiguous, request remittance/review
-> if not found, explain and keep checking if policy allows
```

Selene says:

```text
Thanks - I'll check the bank and payment records now. If you have the payment reference, send it through and I'll match it faster.
```

If found:

```text
I found the payment. It came through yesterday with reference INV-1024. I've matched it to your invoice and marked it paid.
```

If not found:

```text
I can't see that payment in the bank or payment records yet. Please send the remittance or payment reference, and I'll keep checking.
```

## 16. Bank Reconciliation

Bank reconciliation proves bank transactions match accounting/source evidence.

### Reconciliation Sources

```text
bank feed transactions
payment confirmations
AP payments
AR receipts
Payroll payments
refunds
tax payments
loan/lease payments
credit card settlements
bank fees
merchant fees
chargebacks
interest
FX movements
manual journal entries
```

### Reconciliation Statuses

```text
Unmatched
CandidateMatched
Matched
PartiallyMatched
Ambiguous
Exception
Reconciled
Reversed
Archived
```

### Reconciliation Packet

```text
BankReconciliationPacket:
  reconciliation_id
  bank_transaction_id
  bank_account_id
  source_owner
  source_document_ref
  journal_ref
  amount_bank
  amount_source
  variance
  variance_reason
  match_confidence
  reconciliation_status
  reconciled_by
  reconciled_at
  audit_ref
```

### Common Reconciliation Matches

```text
supplier payment -> AP bill
customer receipt -> AR invoice
payroll payment -> payrun
tax payment -> tax liability
loan payment -> debt schedule
lease payment -> lease schedule
bank fee -> bank fee expense
interest received -> interest income
interest paid -> interest expense
merchant settlement -> POS/card clearing
```

## 17. Reconciliation Exceptions

Selene must create exceptions for unresolved bank items.

Exception types:

```text
unmatched_receipt
unmatched_payment
duplicate_bank_transaction
amount_mismatch
bank_fee_unexpected
payment_returned
chargeback
partial_payment_failure
wrong_reference
unknown_payer
unknown_payee
foreign_exchange_difference
settlement_delay
payment_provider_fee_variance
```

Selene says:

```text
I found a bank transaction I can't match. It may be a customer payment, but the reference is unclear. I've put it in the unmatched receipt queue for review.
```

## 18. Chargebacks and Reversals

Banking owns chargeback/reversal evidence.

AR/POS/Logistics determine customer and source context.

Accounting posts reversal/fees.

### Chargeback Fields

```text
chargeback_id
bank_transaction_id
payment_provider_ref
customer_candidate
invoice_candidate
amount
currency
reason_code
fee_amount
evidence_deadline
status
source_evidence_refs
audit_ref
```

Statuses:

```text
Opened
EvidenceRequired
EvidenceSubmitted
Won
Lost
Reversed
FeeApplied
CustomerBalanceReopened
Closed
```

If chargeback happens:

```text
payment previously marked paid
-> provider reverses/holds funds
-> Banking records chargeback
-> AR reopens exposure or dispute status
-> POS/Logistics evidence requested if goods/service issue
-> Accounting posts reversal/fee
-> customer credit risk updated
```

Selene says:

```text
The card payment was reversed by the provider. I've reopened the invoice review and requested the sale and delivery evidence needed for the chargeback response.
```

## 19. Bank Fees, Card Fees, and Provider Fees

Banking must check fees.

Fee types:

```text
bank transfer fee
merchant processing fee
card fee
chargeback fee
refund fee
international transfer fee
FX spread
settlement fee
direct debit failure fee
account maintenance fee
loan fee
overdraft fee
```

Fee validation:

```text
expected_fee
actual_fee
fee_schedule_ref
payment_provider_ref
variance
variance_percent
requires_review
```

Abnormal fee packet:

```text
PaymentFeeExceptionPacket:
  fee_id
  bank_transaction_id
  provider_ref
  expected_fee
  actual_fee
  variance
  reason_candidate
  review_required
  audit_ref
```

Selene says:

```text
The card processing fee on this settlement is higher than expected. I've opened a review before posting the final fee allocation.
```

## 20. Credit Card Settlement Boundary

Document 6 will handle company credit cards, employee spend, receipts, reimbursements, and card budgets.

Document 5 handles banking/payment proof for card feeds and settlements.

Banking must support:

```text
credit card statement feed
card transaction feed
merchant settlement feed
card payment proof
card fee evidence
chargeback evidence
refund settlement evidence
card reconciliation evidence
```

Document 6 owns:

```text
employee card assignment
personal vs business spend
receipt collection
employee spend limits
executive card monitoring
personal charge recovery
reimbursements
card spend budgets
```

Banking and Document 6 must connect, not merge.

## 21. Refund Payment Execution

Refunds are outgoing payments.

AR may approve refund source truth.

Banking executes payment.

Accounting posts refund.

### Refund Payment Flow

```text
AR refund approved
-> Access validates refund payment authority
-> Finance/Cashflow checks if required
-> Banking creates payment instruction
-> payment sent through provider/bank
-> confirmation received
-> AR updates refund status
-> Accounting posts refund
-> reconciliation completes
```

High-risk refund triggers:

```text
refund to different bank account
large refund
refund after chargeback
refund to high-risk customer
refund without original payment proof
refund after long delay
```

These require stronger approval.

## 22. Payroll Payment Execution

Payroll owns employee pay calculation.

Banking executes approved payroll payment.

Flow:

```text
Payroll commits payrun
-> Payroll creates employee pay instruction
-> Access/Authority validates payroll payment approval
-> Banking sends payment batch/file/API
-> bank confirms payments
-> failures identified per employee
-> Payroll receives payment status
-> Accounting posts/reconciles
-> employees receive confirmation through Delivery
```

Payroll batch may have partial failures.

```text
Some employees paid, some failed.
Failed payments create payroll payment exception.
No global "paid" status unless all required payments confirmed or exceptions handled.
```

## 23. Supplier / Contractor Payment Execution

AP owns bill/payment readiness.

Banking executes.

Flow:

```text
AP approves supplier/contractor bill
-> cashflow check
-> payment instruction
-> authority/step-up
-> bank/provider send
-> confirmation
-> AP marks paid
-> Accounting posts payment
-> reconciliation
```

Supplier bank changes must pause payment until verification passes.

## 24. Tax, Super, Pension, CPF, and Statutory Payments

Tax/Compliance owns obligation.

Banking executes payment.

Examples:

```text
GST/VAT remittance
payroll tax
income tax installment
withholding tax
superannuation
CPF
pension
social insurance
worker compensation premium
```

These are critical obligations and must use latest-safe-start timing.

## 25. Loans, Leases, and Debt Payments

Debt/Treasury future document will own loan/lease obligation schedule.

Banking executes payment.

Accounting posts principal/interest.

Finance/Cashflow monitors reserve/cash impact.

Banking must support:

```text
loan repayment
lease payment
vehicle finance
equipment finance
mortgage/property loan
interest payment
principal payment
balloon payment
early repayment
late fee evidence
```

Critical timing law applies.

## 26. Cash Reserve Check Before Payment

Finance/Cashflow owns reserve rule.

Banking must not send payment if cashflow gate blocks it.

### Reserve Check Result

```text
cashflow_check_ref
available_cash
minimum_reserve_required
forecast_cash_after_payment
reserve_breach
approval_required
override_status
```

If reserve breach:

```text
Banking cannot send payment unless emergency override approval exists.
```

Selene says:

```text
This payment would take cash below the current reserve. I need the required emergency approval before I can send it.
```

## 27. Bank Account Changes

Bank account changes are high risk.

Types:

```text
company bank account change
supplier bank detail change
employee payroll bank account change
customer refund bank account change
shareholder dividend bank detail change
contractor bank detail change
```

Rules:

```text
step-up required
authority required
old details retained
new details verified where provider supports it
effective date captured
payment hold if suspicious
audit required
no raw details in logs
```

Employee bank change belongs to Payroll self-service, but Banking may validate/pay.

Supplier bank change belongs to AP/Supplier, but Banking may validate/pay.

Refund bank change belongs to AR/Refund, but Banking may validate/pay.

## 28. Fraud and Payment Risk

Selene must detect payment risk.

Risk signals:

```text
new payee bank account
bank account changed recently
payment amount unusual
payment outside normal schedule
urgent payment pressure
first payment to supplier
refund to different account
duplicate payment candidate
same bank account used by different suppliers
unusual country/currency
payment just below approval threshold
employee changed payroll bank near cutoff
customer overpayment refund request
```

Risk outcomes:

```text
allow
require review
require step-up
require dual approval
hold payment
escalate fraud review
```

PH1.D may summarize risk.

Deterministic Risk/Access/Finance owners decide.

## 29. Payment File Export Fallback

Not every bank has API rails.

Selene must support payment file export where required.

Flow:

```text
payment batch approved
-> payment file generated
-> file hash stored
-> authorized user downloads/uploads to bank
-> bank confirmation imported
-> reconciliation
```

Payment file export is still protected.

```text
Exporting payment file requires authority, step-up where required, audit, and secure handling.
```

## 30. International and Multi-Currency Payments

Banking must support foreign currency payments.

Fields:

```text
transaction_currency
settlement_currency
functional_currency
exchange_rate_ref
estimated_fee
actual_fee
fx_gain_loss_ref
payment_provider_rate_ref
settlement_date
```

Rules:

```text
No FX posting without rate evidence.
Estimated FX is not final truth.
Final bank/provider settlement updates accounting.
FX variance creates accounting entry.
```

Document 12 expands multi-currency and consolidation.

## 31. Accounting Handoff

Banking sends evidence to Accounting.

### Payment Journal Examples

Supplier payment:

```text
Debit: Accounts Payable
Credit: Bank
```

Customer receipt:

```text
Debit: Bank
Credit: Accounts Receivable
```

Bank fee:

```text
Debit: Bank Fees Expense
Credit: Bank
```

Chargeback:

```text
Debit: Accounts Receivable / Chargeback Receivable
Debit: Chargeback Fee Expense
Credit: Bank / Card Clearing
```

FX difference:

```text
Debit/Credit: FX Gain/Loss
Debit/Credit: Bank / Payable / Receivable
```

Accounting owns journal posting.

Banking provides proof.

## 32. PH1.D / GPT-5.5 Role

Allowed:

```text
summarize bank reconciliation exception
suggest likely receipt matches
draft payment approval explanation
explain payment failure
summarize bank fee variance
draft customer payment-found response
draft supplier payment confirmation
explain FX difference
```

Forbidden:

```text
execute payment
approve payment
mark paid
allocate ambiguous receipt
validate bank account as final truth
override reserve
change payment details
post journal
```

## 33. PH1.WRITE Wording

PH1.WRITE owns all user-facing banking/payment wording.

Examples:

### Payment Confirmation

```text
The supplier payment has been sent and confirmed by the bank. I've matched it to bill BILL-884 and sent the remittance advice.
```

### Payment Failure

```text
The bank rejected the payment because the account details appear invalid. I've kept the bill unpaid and opened a payment exception for review.
```

### Customer Payment Found

```text
I found the customer payment and matched it to invoice INV-1024. The invoice is now marked paid.
```

### Ambiguous Match

```text
I found a payment for that amount, but it could belong to more than one invoice. I need the remittance reference before I apply it.
```

## 34. Audit Requirements

Banking audit must record:

```text
audit_event_id
actor_id
action
bank_account_id
payment_instruction_id
payment_confirmation_id
bank_transaction_id
source_owner
source_document_ref
old_value_ref
new_value_ref
amount
currency
provider_ref
payment_rail
approval_refs
step_up_refs
cashflow_check_ref
simulation_ref
bank_response_ref
failure_reason
journal_ref
reconciliation_ref
timestamp
company_id
legal_entity_id
country
reason_code
```

No raw credentials.

No raw full bank account numbers.

No hidden bank-provider responses.

No unaudited payment attempt.

## 35. Failure Branches

### Bank API Unavailable

```text
Payment not sent.
Provider exception created.
If critical, escalate timing risk.
If allowed, use alternate provider/file export.
```

### Payment Partially Failed

```text
Mark successful items confirmed.
Create exceptions for failed items.
Do not mark full batch paid.
```

### Duplicate Payment Risk

```text
Block payment.
Show suspected prior payment.
Require review.
```

### Wrong Bank Account Suspected

```text
Hold payment.
Require bank detail verification.
Notify source owner.
```

### Receipt Ambiguous

```text
Do not allocate.
Create unmatched receipt review.
Ask for remittance/proof.
```

### Chargeback Received

```text
Reopen AR exposure or dispute.
Request source evidence.
Post reversal/fee through Accounting.
```

### Cash Reserve Breach

```text
Block payment unless emergency override approved.
```

### Approval Expires

```text
Payment instruction returns to pending approval or cancelled depending policy.
No bank send.
```

## 36. Required Logical Packets

Future logical packets:

```text
BankAccountPacket
BankProviderConnectionPacket
PaymentRailPacket
PaymentInstructionPacket
PaymentApprovalRequirementPacket
PaymentStepUpPacket
PaymentSubmissionPacket
PaymentConfirmationPacket
PaymentFailurePacket
BankTransactionPacket
BankReceiptPacket
CashApplicationCandidatePacket
UnmatchedReceiptReviewPacket
BankReconciliationPacket
PaymentFeeExceptionPacket
ChargebackPacket
RefundPaymentPacket
PayrollPaymentBatchPacket
SupplierPaymentPacket
TaxPaymentPacket
DebtPaymentPacket
ReserveCheckPacket
FraudRiskSignalPacket
PaymentFileExportPacket
FXSettlementPacket
BankingAuditEvidencePacket
```

Codex must later map these to repo equivalents.

Do not claim they currently exist.

## 37. Example - Supplier Payment With Dual Approval

```text
AP approves supplier bill for AUD 140,000.
Finance cashflow check passes.
Approval matrix requires CEO + CFO.
Harry CEO approves with Face ID.
Bob CFO approves with Face ID.
Banking sends payment through bank API.
Bank confirms.
Accounting posts payment journal.
Banking reconciles transaction.
AP marks bill paid.
```

Selene says:

```text
The supplier payment has been approved by Harry and Bob, sent through the bank, and confirmed. I've matched it to the supplier bill and reconciliation evidence is ready.
```

## 38. Example - Customer Direct Bank Deposit

```text
Bank feed shows AUD 5,000 received.
Reference: INV-1024.
Customer AR invoice INV-1024 open for AUD 5,000.
Match confidence: high.
AR allocates receipt.
Accounting posts receipt.
Banking reconciles.
```

Selene says:

```text
I found the bank deposit for INV-1024 and matched it to the customer invoice. The invoice is now paid and reconciled.
```

## 39. Example - Ambiguous Same-Amount Deposit

```text
Bank feed shows AUD 5,000 received.
Reference: Payment.
Two customers have open invoices for AUD 5,000.
Match confidence: ambiguous.
```

Selene result:

```text
Do not allocate.
Create unmatched receipt review.
Ask for remittance/proof.
```

Selene says:

```text
I found a $5,000 payment, but it could match two different customers. I need a remittance reference before I apply it.
```

## 40. Example - Payment Fails Near Loan Due Date

```text
Loan payment due Friday.
Bank rejects payment Wednesday.
Funds must clear by Friday.
Critical payment risk created.
Selene escalates to CFO/CEO.
Alternate payment route considered.
```

Selene says:

```text
The bank rejected the loan payment, and it must clear by Friday. I've escalated this as a critical payment risk and prepared an alternate payment option for approval.
```

## 41. What Must Not Happen

```text
no bank API call without authority/simulation/audit
no payment marked paid without bank/provider confirmation
no receipt allocated by guessing
no ambiguous receipt auto-matched
no payment batch marked fully paid after partial failure
no supplier/employee/customer bank detail change without protected verification
no raw bank credentials in logs
no PH1.D/GPT-5.5 payment execution
no Adapter or Desktop/iPhone payment authority
no cash reserve breach without emergency approval
no duplicate payment ignored
no chargeback ignored
no abnormal bank/card fee silently posted
no payment file export without authority and audit
no final FX posting without settlement evidence
no implementation from this document alone
```

## 42. Future Simulation Targets

```text
SIM_BANK_001_bank_connection_feed_activation
SIM_BANK_002_supplier_payment_dual_approval_bank_api
SIM_BANK_003_payment_provider_failure
SIM_BANK_004_partial_payroll_batch_failure
SIM_BANK_005_customer_direct_bank_deposit_high_confidence_match
SIM_BANK_006_customer_direct_bank_deposit_ambiguous_match
SIM_BANK_007_customer_says_paid_bank_search
SIM_BANK_008_chargeback_reopens_ar_exposure
SIM_BANK_009_abnormal_bank_fee_review
SIM_BANK_010_cash_reserve_blocks_payment
SIM_BANK_011_critical_loan_payment_bank_failure_escalation
SIM_BANK_012_payment_file_export_fallback
SIM_BANK_013_fx_payment_settlement_variance
SIM_BANK_014_supplier_bank_detail_change_payment_hold
SIM_BANK_015_refund_payment_to_different_account_high_risk
```

## 43. Related Addendum

Live/stale bank balance handling, full bank transaction read logic, automatic categorization, bank-native authorization, universal bank account change governance, supplier/customer/company/shareholder bank change flows, spelling/entity confirmation law, and employee financial wellbeing boundaries are defined in SELENE_BANKING_LIVE_TRUTH_ACCOUNT_CHANGES_TRANSACTION_CATEGORIZATION_AUTHORIZATION_ADDENDUM.md and must be read with this document.

## 44. Final Architecture Sentence

Selene Banking + Payment Rails + Reconciliation is the governed money-movement proof engine: it connects to banks and payment providers, receives bank feeds, accepts only approved payment instructions from canonical source owners, enforces authority, step-up, cashflow, payment-rail timing, simulation, and audit gates, sends payments only through approved rails, records provider confirmation or failure, matches direct deposits to the correct customer and invoice, rejects ambiguous reconciliation guesses, detects fees, chargebacks, reversals, and payment failures, hands accounting proof to the General Ledger, and makes Selene's financial system real-time, bank-aware, and safe enough to move money without humans babysitting every transaction.
