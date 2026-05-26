# Selene Accounts Receivable + Invoices + Debtor Chasing + Collections Master Design

```text
DOCUMENT TYPE:
MASTER DESIGN / ACCOUNTS RECEIVABLE + CUSTOMER INVOICES + RECEIPTS + DEBTOR CHASING + COLLECTIONS

STATUS:
MASTER_DESIGN
NOT_RUNTIME_IMPLEMENTATION
PENDING_GRAND_ARCHITECTURE_RECONCILIATION

PURPOSE:
Define Selene's Accounts Receivable system: customer invoices, sales invoices, POS/customer payment handoff, receipts, payment allocation, debtor chasing, collections, payment plans, customer credit control, 30/60/90+ aging, invoice disputes, credit notes, refunds, Selene-to-Selene invoice delivery, cashflow collection activation, AR accounting handoff, reminders, delivery, audit, and protected access boundaries.
```

## 0. Authority and Scope

AGENTS.md controls execution.

This is a docs-only master design.

No runtime code was changed.

This document does not authorize implementation.

This document is Finance/Accounting Design Batch 2. It defines future Accounts Receivable architecture only. It does not implement AR, Customer, Sales/POS, Banking, Payment Provider, Accounting, Tax, Access, Reminder, Delivery, Desktop, iPhone, Adapter, packets, migrations, tests, or runtime state.

Current repo truth does not prove a complete runtime Accounts Receivable, invoice, receipt, customer payment, payment allocation, collections, payment link, recurring invoice, refund, credit note, banking, or payment provider engine. This document is a future master design pending Grand Architecture Reconciliation.

Future implementation requires repo-truth activation, approved file scope, tests, backend evidence, Access/Governance proof, PH1.BCAST/DELIVERY proof, PH1.REM proof, PH1.WRITE proof, payment provider proof where relevant, simulation proof, audit proof, and JD live acceptance where visible.

## 1. Executive Target

Accounts Receivable is Selene's system for managing money owed to the company.

Old AR:

```text
human creates invoice
human emails invoice
human forgets to follow up
customer pays late
human checks bank manually
human matches payment later
human sends awkward reminder
cashflow suffers
everyone acts surprised
```

Selene AR:

```text
invoice is created from real sales/service evidence
invoice is sent through governed delivery
due date is tracked automatically
customer payment behavior is monitored
bank receipt is matched automatically
debtor reminders are sent automatically
cashflow risk triggers collection action
payment plans are managed
disputes are tracked
collections escalate by policy
Accounting posts the correct entries
Finance sees real-time AR and cashflow impact
```

The target:

```text
no forgotten invoices
no lost receipts
no unpaid customers hiding quietly
no late debtor chasing
no cashflow surprise
no invoice sent without source truth
no customer payment left unmatched
no refund or credit note without authority
no debtor harassment outside policy
```

Selene should be friendly first, persistent second, firm third. Professionally persistent and evidence-backed.

## 2. Master Law

```text
Every receivable must trace to real source evidence.

Every invoice must have customer, amount, tax treatment, due date, payment terms, and delivery evidence.

Every customer payment must be matched, allocated, reconciled, and audited.

Every overdue invoice must trigger the correct reminder, collection, escalation, or payment-plan flow.

Every AR action must preserve customer privacy, access, delivery protocol, accounting truth, and audit.

PH1.D/GPT-5.5 may help explain, draft, classify, and summarize.

PH1.D/GPT-5.5 must not create final AR truth, approve refunds, write off debt, change customer credit terms, or execute financial actions.
```

## 3. Owner Split

### Accounts Receivable owns

```text
customer invoice lifecycle
sales invoice lifecycle
customer account balance
receipts
payment allocation
debtor aging
payment terms
overdue status
customer statements
payment plans
collections state
credit notes
refund request handoff
customer invoice disputes
AR audit evidence
```

### Sales / POS owns

```text
sales event
order event
subscription event
customer purchase
refund trigger evidence
sales commission trigger evidence
POS receipt event
customer transaction evidence
```

Sales/POS creates source evidence. AR creates invoice/receivable truth.

### Delivery / BCAST owns

```text
invoice delivery
statement delivery
payment reminder delivery
payment link delivery
collection notice delivery
customer receipt confirmation delivery
Selene-to-Selene financial document transport
```

AR owns invoice/reminder source truth. Delivery sends.

### PH1.REM owns

```text
due-date reminders
pre-due reminders
overdue follow-ups
promise-to-pay reminders
payment-plan installment reminders
collection escalation timers
```

### Banking / Payment owns

```text
customer payment receipt evidence
bank feed
payment provider confirmation
card settlement
payment link result
failed payment result
reconciliation proof
```

### Accounting owns

```text
AR journal posting
revenue journal
tax liability journal
receipt journal
credit note journal
bad debt/write-off journal
refund journal
```

### Finance / Cashflow owns

```text
cashflow forecast impact
collection priority
debtor risk
credit exposure
customer credit policy
cash shortfall response
```

### Tax / GST / VAT owns

```text
invoice tax treatment
GST/VAT/sales tax rate
tax invoice validity
country/region invoice requirements
tax reporting treatment
```

### Access / Governance owns

```text
who can create invoices
who can send invoices
who can see customer balances
who can approve credit terms
who can approve payment plans
who can approve credit notes
who can approve refunds
who can write off debt
who can place customer on credit hold
who can export AR records
```

### PH1.WRITE owns

```text
invoice summaries
payment reminder wording
debtor chasing wording
collection escalation wording
payment-plan wording
invoice dispute wording
refund/credit note explanations
cashflow collection summaries
```

### PH1.D / GPT-5.5 may assist

```text
draft polite reminders
draft firmer overdue messages
summarize invoice dispute
explain customer balance
suggest likely payment allocation
classify debtor risk as proposal
draft payment plan wording
draft Selene-to-Selene message summaries
```

But must not:

```text
approve refund
approve credit note
write off debt
change payment terms
create final invoice truth
accept tax treatment
send messages directly
decide customer credit limit
allocate payment as final truth
```

## 4. AR Scope

Selene AR must support:

```text
customer invoices
sales invoices
service invoices
subscription invoices
milestone invoices
progress claims
recurring invoices
POS receipt handoff
customer statements
receipts
payment allocation
partial payments
overpayments
underpayments
payment plans
installment collections
early-payment discounts
late-payment fees where policy allows
credit notes
refunds
bad debt write-off requests
collections escalation
customer credit limits
customer credit holds
debtor aging
Selene-to-Selene invoice delivery
```

AR must not own:

```text
bank transfer execution
payment provider truth
general ledger posting truth
tax law truth
sales event truth
customer identity master truth if CRM owns it
delivery mechanics
reminder timing
protected approval
```

## 5. Customer Setup

Before invoices can run properly, Selene must know the customer.

### Customer profile fields

```text
customer_id
legal_name
trading_name
customer_type
country
region
billing_address
shipping_address
tax_registration_number
customer_contact
email
phone
preferred_delivery_channel
payment_terms
credit_limit
credit_status
default_currency
default_tax_treatment_ref
account_manager_id
customer_status
created_at
approved_by
audit_ref
```

### Customer status

```text
Draft
PendingVerification
Active
CreditHold
PaymentPlan
Collections
Suspended
Archived
Blocked
```

### Customer risk fields

```text
payment_history_score
average_days_to_pay
overdue_balance
credit_limit_used
dispute_count
writeoff_history
high_value_customer
strategic_customer
cashflow_priority
collections_risk_level
```

## 6. Invoice Creation

Invoices should be created from source evidence, not casual manual guesses.

Sources:

```text
sales order
POS sale
subscription billing cycle
service completion
delivery completion
project milestone
contract milestone
time-and-materials approved work
manual invoice request with authority
Selene-to-Selene commercial event
```

### Invoice fields

```text
invoice_id
tenant_id
company_id
legal_entity_id
customer_id
invoice_number
invoice_date
due_date
payment_terms
currency
line_items
quantity
unit_price
discount
tax_code_ref
tax_amount
amount_net
amount_gross
source_event_refs
delivery_method
invoice_status
payment_status
accounting_status
audit_ref
```

### Invoice line fields

```text
invoice_line_id
invoice_id
item_or_service_ref
description
quantity
unit_price
discount
tax_code_ref
account_mapping_ref
cost_center_ref
project_ref
department_ref
location_ref
source_evidence_ref
```

### Invoice source validation

Before invoice is issued:

```text
customer active
billing details complete
tax treatment valid
invoice number unique
source evidence valid
line totals correct
payment terms valid
currency valid
delivery channel valid
approval passed if required
audit ready
```

If missing:

```text
InvoiceExceptionPacket:
  invoice_id
  missing_field
  source_owner
  required_action
  audit_ref
```

## 7. Invoice States

```text
Draft
PendingEvidence
PendingApproval
Approved
Sent
Delivered
Viewed
PartiallyPaid
Paid
Overdue
Disputed
PaymentPlanActive
Credited
Refunded
WrittenOff
Cancelled
Archived
```

Invoice status must not be based only on a message being sent.

Delivery status and payment status are separate.

```text
invoice_status = accounting/commercial invoice lifecycle
delivery_status = whether customer received invoice
payment_status = whether money was received and allocated
```

## 8. Invoice Delivery

Invoices may be delivered by:

```text
email
SMS link
customer portal
PDF
secure document link
API
Selene-to-Selene company delivery
postal/manual export where required
```

Delivery flow:

```text
AR prepares invoice
PH1.WRITE prepares customer-safe wording
PH1.BCAST / PH1.DELIVERY sends invoice
delivery receipt captured where available
AR updates delivery status from delivery evidence
PH1.REM schedules follow-up based on due date
```

AR must not directly send messages.

Delivery must not decide invoice truth.

### Invoice delivery packet

```text
InvoiceDeliveryRequestPacket:
  delivery_id
  invoice_id
  customer_id
  recipient_contact_ref
  delivery_channel
  message_ref
  invoice_artifact_ref
  payment_link_ref
  requires_receipt
  status
  audit_ref
```

## 9. Selene-to-Selene Invoice Delivery

If both companies use Selene, invoices should move system-to-system instead of email/PDF where supported.

Flow:

```text
Seller Selene creates invoice.
Seller Selene sends SeleneToSeleneInvoicePacket.
Buyer Selene receives and validates invoice.
Buyer Selene matches PO/contract/goods/service if applicable.
Buyer Selene creates AP candidate.
Buyer Selene sends received/accepted/disputed/payment-status response.
Seller Selene updates AR status.
Bank/payment confirmation closes loop where integrated.
```

### Selene-to-Selene invoice packet

```text
SeleneToSeleneInvoicePacket:
  sender_company_id
  receiver_company_id
  seller_invoice_id
  buyer_reference_candidate
  customer_id
  invoice_lines
  tax_details
  payment_terms
  due_date
  payment_link_or_instruction_ref
  source_evidence_refs
  signature_or_auth_proof
  delivery_status
  audit_ref
```

Supported response packets:

```text
InvoiceReceivedPacket
InvoiceAcceptedPacket
InvoiceDisputedPacket
PaymentScheduledPacket
PaymentSentPacket
PaymentConfirmedPacket
CreditNoteRequestedPacket
```

Important:

```text
Selene-to-Selene delivery is transport and evidence exchange.

Seller AR owns seller invoice truth.
Buyer AP owns buyer bill truth.
Banking owns payment confirmation.
Accounting owns ledger posting.
```

No email if direct Selene financial exchange is available and approved.

## 10. Payment Terms And Aging

Selene must support common and custom AR payment terms.

```text
DueOnReceipt
Net7
Net14
Net30
Net45
Net60
Net90
EndOfMonth
SpecificDate
InstallmentPlan
RecurringPlan
CustomTerms
```

Aging buckets:

```text
not_due
due_today
1_30_days_overdue
31_60_days_overdue
61_90_days_overdue
90_plus_days_overdue
120_plus_days_overdue
collections
bad_debt_review
```

AR must track:

```text
customer_id
invoice_id
due_date
days_overdue
amount_outstanding
aging_bucket
last_reminder_at
next_reminder_at
promise_to_pay_date
collection_status
dispute_status
risk_level
```

## 11. Debtor Chasing Protocol

Selene must chase debtors automatically based on policy.

This is not random nagging. This is governed collection workflow.

### Reminder stages

```text
pre_due_friendly
due_today
early_overdue
first_overdue_notice
30_day_notice
60_day_notice
90_day_notice
final_notice
collections_escalation
legal_review_candidate
credit_hold_candidate
```

### Tone progression

```text
friendly
clear
firm
urgent
formal
escalated
```

### Example wording

Pre-due:

```text
Just a friendly reminder that invoice INV-1024 is due soon. You can pay using the secure link here: [link]
```

Early overdue:

```text
Invoice INV-1024 is now overdue. Could you please arrange payment or let us know if there's an issue?
```

30+ days:

```text
Invoice INV-1024 is now over 30 days overdue. Please arrange payment urgently or contact us today so we can resolve this.
```

60+ days:

```text
Invoice INV-1024 remains unpaid after 60 days. Unless payment or a payment arrangement is received, this may be escalated under your account terms.
```

90+ days:

```text
Invoice INV-1024 is now over 90 days overdue. This requires immediate resolution. Please arrange payment or contact us today to avoid further action under the account terms.
```

Production Selene uses persistent, professional, documented pressure.

## 12. Cashflow-Triggered Collections

Cashflow management must start by collecting money owed, not lazily delaying outgoing payments.

When Finance/Cashflow detects a potential shortfall, AR must provide collection opportunities.

Cashflow protocol:

```text
1. predict cash shortage
2. identify collectible receivables
3. chase overdue invoices
4. chase due-soon high-value invoices
5. prioritize customers likely to pay quickly
6. offer payment links
7. offer payment plans if allowed
8. alert account managers
9. consider credit hold for risky customers
10. only then manage outgoing payments
```

AR collection ranking:

```text
amount_outstanding
days_overdue
customer_payment_history
promise_to_pay_history
likelihood_to_pay
customer importance
relationship owner
payment method available
dispute status
cash urgency
```

Example:

```text
Cash gap forecast in 14 days: $40,000
Outstanding invoices: $120,000
Selene identifies top 10 invoices most likely to be collected quickly.
Selene sends reminders, offers payment links, and alerts account managers.
```

This connects to future Cashflow Forecasting + Payment Priority Intelligence. That future document is not written in this batch.

## 13. Payment Links And Payment Methods

Selene should make it easy for customers to pay.

Payment options may include:

```text
bank transfer
card payment
direct debit
ACH/SEPA/BACS/FPS/EFT where supported
payment provider link
customer portal
Selene-to-Selene payment request
installment plan
manual remittance advice
```

Payment link fields:

```text
payment_link_id
invoice_id
customer_id
amount_due
currency
expiry_at
allowed_payment_methods
provider_ref
status
audit_ref
```

Payment link statuses:

```text
Draft
Active
Clicked
Paid
Expired
Cancelled
Failed
```

Payment provider owns payment confirmation.

AR owns invoice payment status after confirmation/allocation.

## 14. Receipt And Payment Allocation

When money arrives, Selene must match it to invoices.

Sources:

```text
bank feed
payment provider
card settlement
customer remittance advice
Selene-to-Selene payment confirmation
manual receipt with authority
```

Receipt fields:

```text
receipt_id
customer_id_candidate
bank_transaction_ref
payment_provider_ref
amount
currency
received_at
payer_name
reference_text
invoice_reference_candidate
allocation_status
audit_ref
```

Allocation statuses:

```text
Unmatched
Matched
PartiallyMatched
MultipleInvoiceMatch
Overpayment
Underpayment
CustomerUnknown
Disputed
ManualReviewRequired
Allocated
Reconciled
```

Matching logic:

```text
invoice number
customer name
amount
payment reference
bank account
payment link reference
remittance advice
customer history
Selene-to-Selene confirmation
```

GPT-5.5 may suggest likely match.

AR/Banking deterministic rules decide final allocation.

## 15. Partial Payments, Overpayments, Underpayments

### Partial payment

```text
Customer pays less than invoice amount.
Invoice becomes PartiallyPaid.
Remaining balance stays open.
Reminder schedule updates.
```

### Overpayment

```text
Customer pays more than invoice amount.
Selene creates:
- unapplied customer credit, or
- refund candidate, or
- allocation to other open invoices.
```

### Underpayment

```text
Customer pays short.
Selene checks:
- discount allowed?
- dispute?
- bank fee?
- customer error?
- write-off threshold?
```

Small balance write-off requires policy.

Large write-off requires authority.

## 16. Payment Plans And Installment Collections

Selene must support customers paying over time.

Payment plan fields:

```text
payment_plan_id
customer_id
invoice_ids
total_amount
installment_amount
frequency
start_date
end_date
next_due_date
number_of_installments
interest_or_fee_if_allowed
approval_required
customer_acceptance_ref
status
audit_ref
```

Payment plan states:

```text
Draft
PendingApproval
Offered
Accepted
Active
MissedInstallment
Renegotiation
Completed
Defaulted
Cancelled
Archived
```

Example:

```text
Customer owes $12,000.
Selene offers 4 monthly installments of $3,000 if company policy allows.
Customer accepts.
Selene schedules reminders and payment links.
Missed installment escalates.
```

Payment plans affect cashflow forecast.

Finance/Cashflow must receive the expected receipt schedule.

## 17. Invoice Disputes

Customers may dispute invoices.

Dispute reasons:

```text
wrong amount
wrong item
service not delivered
goods damaged
delivery not completed
tax incorrect
duplicate invoice
pricing disagreement
contract disagreement
customer claims already paid
partial delivery
quality issue
```

Dispute states:

```text
Opened
EvidenceCollecting
InternalReview
CustomerContacted
AwaitingCustomerResponse
CreditNoteProposed
RefundProposed
Resolved
Rejected
Escalated
Closed
```

Dispute packet:

```text
ARInvoiceDisputePacket:
  dispute_id
  invoice_id
  customer_id
  reason_code
  customer_message_ref
  evidence_refs
  source_owner_required
  proposed_resolution
  status
  audit_ref
```

Selene may draft customer response through PH1.WRITE.

Delivery uses BCAST/DELIVERY.

AR owns dispute state.

## 18. Credit Notes

A credit note reduces what customer owes.

Credit note reasons:

```text
returned goods
overbilling
pricing correction
tax correction
discount after invoice
service failure
damaged goods
cancellation
goodwill credit
duplicate invoice correction
```

Credit note lifecycle:

```text
credit note requested
-> authority check
-> source evidence checked
-> tax treatment checked
-> credit note created
-> customer notified
-> AR balance reduced
-> accounting posted
-> audit
```

Credit note fields:

```text
credit_note_id
invoice_id
customer_id
reason_code
amount_net
tax_amount
amount_gross
currency
approval_ref
tax_treatment_ref
accounting_ref
delivery_status
audit_ref
```

Credit notes require authority if above threshold or outside policy.

## 19. Refunds

Refunds are outgoing payments linked to AR.

Refund flow:

```text
refund requested
-> customer credit/overpayment verified
-> authority check
-> cashflow check if material
-> payment instruction created
-> Banking/Payment executes
-> Accounting posts refund
-> AR balance updated
-> customer notified
-> audit
```

Refund must not bypass payment controls.

Refund statuses:

```text
Draft
PendingApproval
Approved
PaymentInstructionCreated
SentToBank
Paid
Failed
Cancelled
Archived
```

Refund protected actions:

```text
approve refund
send refund payment
change refund bank details
refund above threshold
refund to different payee
```

## 20. Bad Debt And Write-Off

Some debts may become uncollectable.

Bad debt flow:

```text
invoice overdue
collections exhausted
dispute resolved or closed
customer risk reviewed
write-off proposal prepared
authority check
tax/accounting treatment checked
write-off approved
Accounting posts bad debt entry
customer account updated
audit
```

Write-off requires:

```text
reason
collection history
approval authority
tax treatment
accounting treatment
audit
```

Selene must not write off debt casually because a customer is difficult.

## 21. Credit Control And Customer Holds

Selene must manage customer credit risk.

Credit control signals:

```text
overdue invoices
credit limit exceeded
repeated late payment
disputes
failed payments
collections status
cashflow priority
customer risk score
```

Actions:

```text
warn account manager
block new credit sale
require upfront payment
reduce credit limit
place account on hold
escalate to finance
restore credit after payment
```

Protected actions:

```text
change credit limit
place customer on hold
block customer ordering
write off debt
send formal collection notice
```

Access/Authority required.

## 22. Customer Statements

Selene should generate customer statements.

Statement fields:

```text
statement_id
customer_id
statement_date
opening_balance
invoices
payments
credits
refunds
closing_balance
aging_summary
delivery_status
audit_ref
```

Statement delivery:

```text
AR prepares statement.
PH1.WRITE summarizes.
PH1.BCAST/DELIVERY sends.
PH1.REM schedules follow-up if overdue.
```

If customer uses Selene, send through Selene-to-Selene statement packet.

## 23. Recurring Invoices And Subscriptions

Selene must support recurring AR.

Examples:

```text
monthly subscription
service contract
rental income
retainer
membership fee
maintenance contract
software license
scheduled customer installment
```

Recurring invoice fields:

```text
recurring_invoice_id
customer_id
contract_ref
frequency
amount
currency
tax_code_ref
start_date
end_date
next_invoice_date
auto_send_allowed
approval_required
status
audit_ref
```

Flow:

```text
schedule active
-> invoice draft generated
-> source/contract checked
-> tax checked
-> approval if required
-> invoice sent
-> due date tracked
-> payment collected
```

## 24. Rental Income Link

Rental income from property assets belongs mainly to a future Real Estate / Property Assets document, but AR must support the receivable.

Example:

```text
Property lease generates monthly rent invoice.
Real Estate owner provides lease terms.
AR creates invoice.
Banking matches tenant payment.
Accounting posts rental income.
```

This links to a future Real Estate + Property Assets + Rental Income + Collateral document. That future document is not written in this batch.

## 25. AR Accounting Handoff

AR sends accounting evidence to GL.

### Invoice journal

```text
Debit: Accounts Receivable
Credit: Revenue
Credit: GST/VAT/Sales Tax Payable where applicable
```

### Receipt journal

```text
Debit: Bank
Credit: Accounts Receivable
```

### Credit note journal

```text
Debit: Revenue / Sales Returns / Tax Payable adjustment
Credit: Accounts Receivable
```

### Refund journal

```text
Debit: Customer Refund Liability / Revenue Adjustment
Credit: Bank
```

### Bad debt journal

```text
Debit: Bad Debt Expense
Credit: Accounts Receivable
```

Accounting owns posting.

AR owns invoice and receivable truth.

## 26. Tax / GST / VAT In AR

Invoices must satisfy country/region tax rules.

AR must capture:

```text
customer tax registration
seller tax registration
tax code
tax rate
tax amount
tax inclusive/exclusive flag
zero-rated/exempt status
reverse charge where applicable
country/region invoice requirements
tax invoice format
effective-date rule
```

Tax owner validates:

```text
GST/VAT/sales tax treatment
tax invoice validity
tax reporting impact
credit note tax adjustment
refund tax adjustment
```

GPT-5.5 may explain.

It must not invent tax treatment.

## 27. Cash Application And Reconciliation

AR cash application connects to Banking.

Flow:

```text
bank/payment provider receives money
-> Banking creates receipt evidence
-> AR matches receipt to invoice/customer
-> unmatched goes to cash application queue
-> matched receipt updates invoice status
-> Accounting posts receipt journal
-> Banking reconciles bank transaction
```

Unmatched receipts:

```text
unknown customer
wrong reference
overpayment
partial payment
bulk payment
multiple invoice payment
foreign currency mismatch
bank fee deducted
customer paid old account
```

Selene should explain:

```text
I received $8,400 from a customer, but the reference does not match any open invoice. I found three likely matches and need review before allocation.
```

## 28. AR Interaction With Sales / Marketing

If cashflow risk exists, AR may trigger Sales/Marketing opportunities through Finance/Cashflow, not directly mutate campaigns.

Possible actions:

```text
abandoned quote follow-up
renewal reminder
repeat-customer offer
early payment discount campaign
slow-stock clearance
high-margin product push
subscription upgrade outreach
```

Owner split:

```text
AR identifies cash collection need.
Cashflow ranks urgency.
Sales/Marketing owns campaign/action.
PH1.WRITE drafts messages.
BCAST/DELIVERY sends approved outreach.
```

This expands in future Cashflow Forecasting + Payment Priority Intelligence. That future document is not written in this batch.

## 29. Security And Privacy

AR data is sensitive.

Sensitive fields:

```text
customer balance
customer payment history
customer bank/payment data
tax registration
credit limit
credit hold status
disputes
collections notes
customer statements
payment links
refund information
```

Access controls required:

```text
view AR balance
create invoice
approve invoice
send invoice
approve credit note
approve refund
write off debt
change customer payment terms
change customer credit limit
place credit hold
export AR report
view collections notes
```

Step-up may be required for:

```text
refund approval
write-off approval
credit limit change
customer bank/payment detail change
large credit note
formal collections escalation
AR data export
```

## 30. PH1.D / GPT-5.5 Role

Allowed:

```text
draft friendly payment reminders
draft firm overdue notices
summarize invoice disputes
suggest likely payment allocations
explain AR aging
draft customer statement explanation
summarize collections risk
draft payment-plan proposal
translate customer messages
```

Forbidden:

```text
approve refund
approve credit note
write off debt
send collection notice directly
change credit terms
create final invoice truth
accept final tax treatment
allocate receipt as final truth
```

## 31. PH1.WRITE Wording

PH1.WRITE owns customer-facing wording.

### Friendly reminder

```text
Just a friendly reminder that invoice INV-1024 is due soon. You can pay securely here: [link]
```

### Firm overdue

```text
Invoice INV-1024 remains unpaid and is now 30 days overdue. Please arrange payment or contact us today if there is an issue.
```

### Payment plan

```text
We can offer a payment plan for the outstanding balance if that helps. Please review the proposed schedule and confirm whether it works for you.
```

### Dispute acknowledgement

```text
Thanks for letting us know. I've opened a review for this invoice and will check the supporting order, delivery, and pricing details before we follow up.
```

## 32. Audit Requirements

Audit must record:

```text
audit_event_id
actor_id
action
customer_id
invoice_id
receipt_id
payment_id
credit_note_id
refund_id
old_value_ref
new_value_ref
source_evidence_refs
delivery_refs
reminder_refs
payment_provider_refs
bank_confirmation_refs
approval_refs
step_up_refs
tax_rule_refs
journal_refs
timestamp
company_id
legal_entity_id
country
currency
reason_code
```

No raw payment credentials in audit logs.

No hidden reminder history.

No silent write-offs.

## 33. Failure Branches

### Invoice delivery fails

```text
Delivery failure recorded.
AR does not mark invoice delivered.
Alternate delivery path may be used if policy allows.
```

### Customer disputes invoice

```text
Collections paused or modified based on policy.
Dispute case opened.
Evidence gathered.
```

### Payment received but unmatched

```text
Receipt goes to cash application review.
Invoice not marked paid until allocated.
```

### Customer pays less

```text
Invoice becomes partially paid.
Remaining balance stays open unless write-off/discount approved.
```

### Customer overpays

```text
Create unapplied credit or refund candidate.
No refund without approval.
```

### Payment link fails

```text
Payment provider failure recorded.
Customer gets safe retry message.
AR remains unpaid.
```

### Customer ignores reminders

```text
Escalate based on aging policy.
Notify account manager/collections.
Consider credit hold if authorized.
```

### Tax code missing

```text
Invoice cannot be finalized or sent if tax treatment required.
Route to Tax owner.
```

### Credit note requires approval

```text
Credit note remains pending until authority passes.
```

### Refund payment fails

```text
Refund remains unpaid.
Banking/Payment failure proof captured.
Customer notified only through approved wording.
```

## 34. Required Logical Packets

Future logical packets:

```text
CustomerProfilePacket
CustomerCreditPolicyPacket
CustomerInvoicePacket
InvoiceLinePacket
InvoiceDeliveryRequestPacket
SeleneToSeleneInvoicePacket
InvoiceReceivedPacket
InvoiceAcceptedPacket
InvoiceDisputedPacket
PaymentLinkPacket
CustomerReceiptPacket
CashApplicationPacket
PaymentAllocationPacket
ARReminderPlanPacket
DebtorAgingPacket
CollectionsEscalationPacket
PaymentPlanPacket
InvoiceDisputePacket
CreditNotePacket
RefundRequestPacket
CustomerStatementPacket
RecurringInvoicePacket
ARAccountingHandoffPacket
ARAuditEvidencePacket
```

Codex must later map these to repo equivalents.

Do not claim they currently exist.

## 35. Example End-To-End Flow — Customer Invoice Paid On Time

```text
Sales order completed.
AR creates customer invoice.
Tax treatment validated.
Invoice sent through PH1.BCAST / PH1.DELIVERY.
PH1.REM schedules pre-due follow-up.
Customer pays through payment link.
Payment provider confirms.
AR allocates receipt to invoice.
Accounting posts receipt.
Banking reconciles.
Invoice marked paid.
```

Selene says:

```text
Invoice INV-1024 has been paid and reconciled. The receipt matched the payment link, and Accounting has the posting evidence.
```

## 36. Example End-To-End Flow — Debtor Chasing

```text
Invoice INV-2048 due date passes.
No payment received.
Day 1 overdue: friendly reminder.
Day 14 overdue: firm reminder.
Day 30 overdue: account manager notified.
Day 60 overdue: collections path starts.
Day 90 overdue: credit hold and formal review candidate.
```

Selene says internally:

```text
Customer has AUD 42,000 overdue, average payment delay 38 days, and two broken promises to pay. I recommend credit hold review and direct account manager follow-up.
```

Customer-facing wording stays professional.

## 37. Example End-To-End Flow — Cashflow Risk Activates Collections

```text
Finance forecasts cash gap in 21 days.
AR identifies $180,000 outstanding.
Selene ranks customers likely to pay quickly.
Selene sends approved reminders with payment links.
Account managers get top debtor task list.
Cashflow forecast updates as receipts arrive.
```

Selene says to Finance:

```text
I found $180,000 in open receivables. The top 12 invoices could close the projected cash gap if collected within 14 days. I've prepared collection actions and account-manager follow-ups.
```

## 38. Example End-To-End Flow — Selene-to-Selene Invoice

```text
ABC Wines Selene issues invoice to XYZ Retail Selene.
Invoice packet delivered securely.
XYZ Retail Selene validates against purchase order.
XYZ Retail Selene accepts invoice.
XYZ Retail Selene schedules payment.
ABC Wines Selene tracks AR status.
Bank confirms payment.
Both ledgers receive accounting evidence.
```

Selene says:

```text
XYZ Retail's Selene accepted the invoice and scheduled payment for the due date. I'll track the payment confirmation and reconcile it once received.
```

## 39. What Must Not Happen

```text
no invoice without source evidence unless authorized manual invoice path exists
no invoice sent without valid customer and tax treatment where required
no AR message sent directly without BCAST/DELIVERY or approved connector
no debtor reminder based on stale or wrong balance
no harassment or unlawful collection wording
no customer payment marked paid without receipt/payment proof
no unmatched bank receipt silently allocated
no refund without authority
no credit note without reason and approval where required
no write-off without authority and audit
no customer credit hold without policy/authority
no GPT-5.5 final AR truth
no tax treatment invented by GPT-5.5
no customer payment data exposed without Access
no old AR records erased
no implementation from this document alone
```

## 40. Future Simulation Targets

```text
SIM_AR_001_customer_invoice_creation_from_sales_order
SIM_AR_002_invoice_delivery_and_pre_due_reminder
SIM_AR_003_payment_link_receipt_and_allocation
SIM_AR_004_unmatched_bank_receipt_review
SIM_AR_005_partial_payment_remaining_balance
SIM_AR_006_overpayment_credit_or_refund
SIM_AR_007_30_60_90_debtor_chasing
SIM_AR_008_cashflow_risk_triggers_collection_actions
SIM_AR_009_customer_invoice_dispute
SIM_AR_010_credit_note_approval_and_posting
SIM_AR_011_refund_approval_and_payment
SIM_AR_012_customer_payment_plan
SIM_AR_013_credit_hold_for_overdue_customer
SIM_AR_014_selene_to_selene_invoice_delivery
SIM_AR_015_recurring_invoice_generation
SIM_AR_016_rental_income_invoice_handoff
```

## 41. Final Architecture Sentence

Selene Accounts Receivable + Invoices + Debtor Chasing + Collections is the governed incoming-money engine: it creates invoices from real sales/service/rental/source evidence, delivers invoices through approved channels or Selene-to-Selene exchange, tracks payment terms and 30/60/90+ aging, sends professional debtor reminders, activates collections when cashflow requires it, manages receipts, allocations, payment plans, disputes, credit notes, refunds, credit control, accounting handoffs, tax treatment, access gates, and audit so Selene can collect what the company is owed quickly, lawfully, intelligently, and with humans involved only for approvals, disputes, exceptions, and judgment.

## Related Addendum

Customer type handling, Selene-connected vs non-Selene customer flows, credit control, bank-payment matching, chargebacks, refunds/credit governance, instant invoice resend, delivery/POS boundaries, customer relationship boundaries, and standalone future customer/credit/collections/logistics/POS modules are defined in SELENE_ACCOUNTS_RECEIVABLE_CUSTOMER_TYPES_CREDIT_CONTROL_PAYMENT_MATCHING_COLLECTIONS_ADDENDUM.md and must be read with this document.
