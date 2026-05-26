# Selene Accounts Payable + Supplier Bills + Installments + Scheduled Payments Master Design

```text
DOCUMENT TYPE:
MASTER DESIGN / ACCOUNTS PAYABLE + SUPPLIER BILLS + INSTALLMENTS + SCHEDULED PAYMENTS

STATUS:
MASTER_DESIGN
NOT_RUNTIME_IMPLEMENTATION
PENDING_GRAND_ARCHITECTURE_RECONCILIATION

PURPOSE:
Define Selene's Accounts Payable system: supplier bills, contractor invoices, installment plans, lease payments, recurring payments, supplier payment terms, AP approvals, payment scheduling, cashflow-aware payment control, accounting handoff, banking/payment handoff, tax treatment, supplier communication, disputes, credit notes, and audit.
```

## 0. Authority and Scope

AGENTS.md controls execution.

This is a docs-only master design.

No runtime code was changed.

This document does not authorize implementation.

This document is Finance/Accounting Design Batch 2. It defines future Accounts Payable architecture only. It does not implement AP, Procurement, Receiving, Banking, Payment Provider, Accounting, Tax, Access, Reminder, Delivery, Desktop, iPhone, Adapter, packets, migrations, tests, or runtime state.

Current repo truth does not prove a complete runtime Accounts Payable, supplier bill, contractor invoice, payment rail, banking, payment provider, procurement, receiving, or payment scheduling engine. This document is a future master design pending Grand Architecture Reconciliation.

Future implementation requires repo-truth activation, approved file scope, tests, backend evidence, Access/Governance proof, PH1.BCAST/DELIVERY proof, PH1.REM proof, PH1.WRITE proof, payment provider proof where relevant, simulation proof, audit proof, and JD live acceptance where visible.

## 1. Executive Target

Accounts Payable is Selene's system for managing money the company owes.

Old AP:

```text
supplier emails invoice
human downloads it
human enters bill
human checks PO
human asks manager
human forgets due date
human pays late
human reconciles later
human apologizes to supplier
```

Selene AP:

```text
supplier bill arrives
Selene reads it
Selene matches supplier, PO, goods received, contract, tax, budget, and approval rules
Selene schedules payment automatically
Selene checks cashflow before payment
Selene routes approval to the correct people
Selene pays through approved bank/payment rails
Selene posts accounting entries
Selene reconciles the bank confirmation
Selene stores audit evidence
```

The target:

```text
no missed supplier bills
no forgotten lease payments
no silent duplicate invoices
no wrong supplier bank payments
no payment without authority
no payment that breaks reserve-cash policy unless emergency approval passes
no contractor invoice paid as employee payroll by accident
no AP entry without evidence
```

Plain human version: Selene becomes the accounts payable clerk who never loses invoices, never ignores due dates, and never treats a high-value supplier payment as casual.

## 2. Master Law

```text
Every outgoing liability must have source evidence.

Every supplier bill must be matched, classified, approved, scheduled, paid, and reconciled through governed owners.

Every payment must pass authority, simulation, cashflow policy, payment proof, and audit.

Recurring payments must be scheduled before they are due.

Installments and leases must be managed as payment obligations, not remembered by humans.

AP owns supplier/vendor/contractor bill liability and outgoing payment workflow.

Accounting owns journal posting.

Banking/Payment owns payment execution and confirmation.

Finance/Cashflow owns affordability and payment-priority intelligence.

Access owns who can approve, pay, override, or view sensitive supplier/payment data.

PH1.D/GPT-5.5 may help read, classify, explain, and draft.

PH1.D/GPT-5.5 must not approve or execute payment.
```

## 3. AP Means Accounts Payable, Not Approval Policy

This document uses:

```text
AP = Accounts Payable
```

Access documents may use AP to mean Approval Policy.

That ambiguity must be explicitly handled.

```text
Accounts Payable must not be confused with Access Approval Policy.

When docs/code say AP, Codex must determine whether it means:
- Accounts Payable, or
- Access Approval Policy.

If unclear, mark OWNER_AMBIGUITY_GAP.
```

## 4. Owner Split

### Accounts Payable owns

```text
supplier bill lifecycle
contractor invoice lifecycle
vendor bill lifecycle
bill validation
payment terms
due dates
installments
lease payment schedules
recurring payment schedules
bill dispute state
supplier credit notes
supplier statement matching
AP aging
AP approval routing
payment readiness
payment instruction request
AP audit evidence
```

### Procurement / Purchasing owns

```text
purchase request
purchase order
supplier selection
purchase approval
goods/services receipt
PO matching evidence
```

### Contractor / Contract owner owns

```text
contract terms
approved contractor hours
milestones
scope
contract expiry
contract overrun approval
```

### Finance / Budget owns

```text
budget availability
spend limits
cash reserve policy
payment priority
cashflow warning
over-budget approval
financial governance
```

### Accounting owns

```text
AP liability journal
expense/inventory/asset classification posting
GST/VAT/tax accounting entry
payment journal
credit note journal
reversal/correction posting
```

### Banking / Payment Provider owns

```text
payment rails
bank API
payment file
payment confirmation
payment failure
bank reconciliation evidence
```

### Tax / GST / VAT owner owns

```text
tax code
GST/VAT claimability
withholding treatment
tax invoice validity
country/region tax rules
effective-date tax rule pack
```

### Access / Governance owns

```text
who can view supplier bills
who can view supplier bank details
who can approve AP bills
who can approve payments
who can override cash reserve
who can approve new supplier bank details
who can approve contractor overrun
who can approve urgent payment
who can export AP data
```

### PH1.REM owns

```text
payment due reminders
approval reminder timing
invoice dispute follow-up reminders
supplier statement review reminders
recurring payment reminders
```

### PH1.BCAST / PH1.DELIVERY owns

```text
approval request delivery
supplier payment notice delivery
remittance advice delivery
supplier dispute messages
supplier reminder messages where applicable
Selene-to-Selene AP/AR delivery
```

### PH1.WRITE owns

```text
supplier wording
approval summaries
AP exception explanations
payment confirmation messages
supplier dispute wording
human-readable bill summaries
```

### PH1.D / GPT-5.5 may assist

```text
read invoice text
extract supplier/bill candidates
draft AP explanations
suggest expense category
summarize payment risk
draft supplier dispute message
explain cashflow impact
```

But must not:

```text
approve supplier bill
approve contractor invoice
execute payment
override cash reserve
invent tax treatment
change supplier bank details
post accounting journal
```

## 5. AP Scope

Selene AP must support:

```text
supplier invoices
vendor bills
contractor invoices
installments
lease payments
equipment finance payments
vehicle loan payments
rent
subscriptions
insurance premiums
tax installments
utility bills
purchase order bills
goods received matching
service completion matching
supplier statements
credit notes
debit notes
refunds from suppliers
payment plans
supplier disputes
contractor overrun payments
recurring payments
scheduled payments
```

AP must not own:

```text
customer invoices
customer receipts
employee payroll
employee salary calculation
general ledger posting truth
bank execution truth
tax law truth
budget truth
```

## 6. Supplier / Vendor / Payee Setup

Before AP can safely pay, Selene must know who is being paid.

### Supplier profile fields

```text
supplier_id
legal_name
trading_name
country
region
business_registration_number
tax_registration_number
supplier_type
supplier_category
primary_contact
email
phone
payment_terms
default_currency
default_tax_treatment_ref
default_account_mapping_ref
bank_account_ref
supplier_status
risk_status
created_at
approved_by
audit_ref
```

### Supplier status

```text
Draft
PendingVerification
Active
Suspended
PaymentHold
Archived
Blocked
```

### Supplier bank details

Supplier bank details are high-risk.

Changing supplier bank details requires:

```text
step-up verification
supplier verification policy
approval authority
fraud check where available
old bank history retained
effective date
audit
payment hold if suspicious
```

Selene must never silently change supplier bank account because someone emailed "new bank details." That is not automation. That is volunteering to be robbed with extra steps.

## 7. Bill / Invoice Intake

Bills can arrive from:

```text
email
upload
supplier portal
Selene-to-Selene company delivery
OCR/document intake
API integration
manual supplier entry
recurring schedule
purchase order conversion
contractor submission
```

### Bill intake packet

```text
SupplierBillIntakePacket:
  intake_id
  supplier_candidate
  source_channel
  document_artifact_ref
  invoice_number_candidate
  invoice_date_candidate
  due_date_candidate
  gross_amount_candidate
  tax_amount_candidate
  currency_candidate
  line_items_candidate
  payment_terms_candidate
  purchase_order_candidate
  confidence_summary
  extraction_status
  audit_ref
```

PH1.D/GPT-5.5 may help extract candidates from messy invoice text.

Deterministic AP validation decides what is accepted.

## 8. Duplicate Invoice Detection

Selene must detect duplicate bills before AP accepts them.

Duplicate checks:

```text
same supplier
same invoice number
same amount
same date
same purchase order
same bank/payment reference
same document hash
similar invoice image
similar line items
already paid invoice
cancelled/credited invoice
```

If duplicate suspected:

```text
APDuplicateInvoiceException:
  bill_id
  suspected_duplicate_bill_ids
  match_reason
  risk_level
  action_required: review
```

Selene says:

```text
This looks like a duplicate of invoice INV-884 from the same supplier. I'll hold it for review before it can be approved or paid.
```

## 9. Bill Validation

Before creating AP liability, Selene validates:

```text
supplier exists
supplier active
supplier bank details approved
invoice number present
invoice date valid
due date valid
currency valid
amounts add up
tax amount valid
tax invoice requirements met
purchase order match if required
goods received match if required
service/milestone evidence if required
contractor hours/milestone evidence if contractor
budget line exists if required
account mapping candidate exists
approval route resolvable
no duplicate invoice
```

Validation status:

```text
Valid
PendingSupplierResolution
PendingTaxReview
PendingPOMatch
PendingGoodsReceipt
PendingContractorEvidence
PendingBudgetReview
PendingApproval
Rejected
DuplicateHold
```

## 10. Purchase Order Matching

If a bill relates to a purchase order, AP must match it.

### Matching types

```text
2-way match:
  PO vs invoice

3-way match:
  PO vs invoice vs goods received

service match:
  contract/service order vs invoice vs service completion evidence

contractor match:
  contract/milestone/hours vs invoice
```

### Match fields

```text
purchase_order_id
supplier_id
invoice_id
goods_received_ref
service_completion_ref
contract_ref
line_item_match_status
quantity_match_status
price_match_status
tax_match_status
variance_amount
variance_percent
approval_required
```

If variance exceeds tolerance:

```text
PO_MATCH_VARIANCE_EXCEPTION
```

Selene says:

```text
The supplier bill is $420 higher than the approved purchase order. I need approval before this can continue.
```

## 11. Contractor Invoice Matching

Contractor invoices are AP unless company policy says otherwise.

Contractor invoice matching checks:

```text
contractor profile
contract terms
billing mode
approved hourly rate
approved hours
logged hours
milestone completion
task evidence
manager verification
contract overrun status
invoice amount
invoice period
tax/vendor compliance
access expiry relevance
```

Billing modes:

```text
hourly
lump_sum
milestone
retainer
project_based
site_contractor
agency
```

Example:

```text
Michael submits invoice for 42 hours.
Approved hours = 40.
Selene flags 2-hour overrun.
AP cannot approve payment until contractor overrun approval passes.
```

Contractor payment must not default to Payroll.

## 12. AP Bill Lifecycle

### Bill states

```text
Received
Draft
PendingValidation
PendingMatch
PendingTaxReview
PendingBudgetCheck
PendingApproval
Approved
ScheduledForPayment
PaymentInstructionCreated
SentToBank
Paid
PartiallyPaid
Disputed
OnHold
Credited
Cancelled
Archived
```

### Lifecycle flow

```text
bill received
-> bill intake
-> duplicate check
-> supplier resolution
-> document validation
-> PO/contract/goods/service match
-> tax/GST/VAT validation
-> account/category mapping
-> budget/cashflow check
-> approval routing
-> bill approved
-> payment scheduled
-> payment instruction created
-> bank/payment execution
-> payment confirmation
-> accounting entry posted
-> reconciliation
-> bill closed
```

## 13. Payment Terms: 7 / 14 / 30 / 60 / 90 / Custom

Selene must manage supplier payment terms.

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
InstallmentSchedule
RecurringSchedule
CustomTerms
```

Fields:

```text
payment_terms_id
supplier_id
term_type
days_after_invoice
due_date_rule
early_payment_discount
late_fee_rule
currency
effective_from
effective_to
approved_by
audit_ref
```

AP aging buckets:

```text
not_due
due_today
1_30_days_overdue
31_60_days_overdue
61_90_days_overdue
90_plus_days_overdue
```

Selene must pay suppliers within the required time frame unless cashflow, approval, or policy says otherwise.

## 14. Installments, Lease Payments, and Scheduled Payments

Selene must manage payments that repeat over time.

Examples:

```text
monthly lease payment
vehicle finance payment
equipment loan
rent
insurance premium
software subscription
tax installment
supplier payment plan
loan repayment
asset financing
contractor staged payment
```

### Scheduled payment fields

```text
scheduled_payment_id
counterparty_id
source_contract_ref
source_bill_ref
payment_type
amount
currency
frequency
start_date
end_date
next_due_date
remaining_payments
payment_method_ref
approval_required
cashflow_check_required
auto_pay_allowed
bank_provider_ref
status
audit_ref
```

### Schedule states

```text
Draft
Active
Paused
PendingApproval
DueSoon
DueToday
PaymentReady
PaymentBlocked
Paid
Failed
Completed
Cancelled
Archived
```

### Scheduled payment flow

```text
schedule created
-> approval obtained if required
-> next due date monitored
-> cashflow checked before due
-> approval requested if required
-> payment instruction created
-> bank payment executed
-> confirmation recorded
-> accounting posted
-> next due date calculated
```

### Lease payment example

```text
Lease payment due monthly on the 15th.
Selene checks the lease schedule.
Selene checks cash reserve.
Selene checks approval policy.
Selene prepares payment.
Selene sends approval if required.
Bank confirms payment.
Accounting posts lease expense/liability treatment based on policy.
```

Future Assets / Debt / Lease Liability documents expand lease and asset treatment where asset or liability rules apply. Those documents are not written in this batch.

## 15. Automatic Payment Scheduling

Selene should automatically schedule approved payments.

AP must maintain a payment calendar.

```text
payment_calendar_id
bill_id
supplier_id
due_date
planned_payment_date
amount
currency
cashflow_status
approval_status
payment_status
priority
late_fee_risk
supplier_criticality
audit_ref
```

Payment timing logic:

```text
pay_on_due_date
pay_early_for_discount
pay_now_if_urgent
delay_if_terms_allow
partial_payment_if_allowed
hold_for_dispute
hold_for_cashflow_review
hold_for_approval
```

Selene must not pay early if it harms cashflow unless there is a benefit and approval.

Selene must not pay late without reason, policy, and supplier risk awareness.

## 16. Cashflow Check Before Payment

Before payment, AP must ask Finance/Cashflow:

```text
Can this payment be made without breaching:
- minimum cash reserve,
- payroll funding,
- tax obligations,
- scheduled lease/loan payments,
- emergency cash reserve,
- board cash policy,
- critical supplier priority,
- bank balance,
- expected receipts?
```

Cashflow result:

```text
CashflowCheckPacket:
  payment_id
  amount
  currency
  planned_payment_date
  bank_account_ref
  current_available_cash
  reserve_cash_requirement
  forecast_cash_after_payment
  reserve_breach: true / false
  recommendation:
    - pay_now
    - pay_on_due_date
    - delay_if_allowed
    - partial_payment
    - emergency_override_required
    - finance_review_required
  audit_ref
```

If reserve breach:

```text
Payment cannot proceed normally.
Emergency override requires required AP/Finance/Executive approval.
```

Example:

```text
This payment would take cash below the $12 million reserve. I can route an emergency override request, delay the payment if terms allow, or propose a partial payment.
```

Future Cashflow Forecasting + Payment Priority Intelligence expands cashflow intelligence. That future document is not written in this batch.

## 17. Payment Priority

AP should rank payment urgency.

Priority factors:

```text
due date
late fee
supplier criticality
service cutoff risk
legal obligation
tax obligation
payroll dependency
contract breach risk
early payment discount
supplier relationship
cashflow impact
budget status
board reserve policy
```

Example:

```text
Tax payment due tomorrow outranks non-critical office supplies due in 14 days.

Critical supplier required for production outranks discretionary marketing service.
```

Selene should propose payment priority but Finance/Cashflow owns final payment-priority governance.

## 18. AP Approval Matrix

AP approvals may depend on:

```text
amount
supplier
supplier risk
new supplier
new bank account
department
cost center
budget status
contract type
payment urgency
country
legal entity
payment method
cash reserve impact
asset/capex status
tax risk
```

Approval patterns:

```text
single approver
manager + finance
CFO approval
CEO approval
board approval
2 of 3 directors
CEO + CFO dual approval
chairman approval
department owner + finance controller
```

### Multi-authority example

```text
Payment over $100,000 requires:
- Harry CEO approval
- Bob CFO approval
- both must pass Face ID/fingerprint/passcode step-up
- no payment until both approvals are complete
```

### Approval packet

```text
APPaymentApprovalRequestPacket:
  approval_request_id
  bill_id
  payment_id
  supplier_id
  amount
  currency
  due_date
  payment_reason
  risk_summary
  cashflow_summary_ref
  required_approvers
  approvals_received
  step_up_required
  approval_status
  expires_at
  audit_ref
```

### Step-up rule

High-risk payment approvals require step-up:

```text
Face ID
fingerprint
secure passcode
approved device confirmation
```

Biometric/passcode proves the approver's presence. It does not replace authority.

## 19. Payment Instruction Creation

Once a bill is approved and cashflow allows payment, AP requests payment instruction.

```text
APPaymentInstructionRequestPacket:
  payment_instruction_id
  bill_id
  supplier_id
  payee_bank_ref
  amount
  currency
  payment_date
  payment_method
  approval_refs
  cashflow_check_ref
  accounting_refs
  bank_provider_ref
  status
  audit_ref
```

Payment instruction status:

```text
Draft
PendingApproval
Approved
ReadyForBank
SentToBank
Confirmed
PartiallyFailed
Failed
Cancelled
Reconciled
```

AP prepares payment instruction.

Banking/Payment executes.

Accounting posts.

AP closes bill.

## 20. Payment Execution Boundary

AP must not directly call the bank.

Flow:

```text
AP marks bill ready for payment
-> Finance/Cashflow confirms payment priority
-> Access confirms authority
-> Banking/Payment provider receives payment instruction
-> bank/payment provider executes
-> confirmation returns
-> Accounting posts payment journal
-> Banking reconciles
-> AP bill marked paid
```

Hard rule:

```text
No bank API call from AP alone.

Payment execution requires:
- approved bill
- approved payment instruction
- Access authority
- simulation
- payment provider proof
- audit
```

## 21. Accounting Handoff

AP sends accounting evidence to GL.

### Bill approved journal

```text
Debit: Expense / Inventory / Fixed Asset / Prepayment
Debit: GST/VAT Receivable if claimable
Credit: Accounts Payable
```

### Payment confirmed journal

```text
Debit: Accounts Payable
Credit: Bank
```

### Credit note journal

```text
Debit: Accounts Payable
Credit: Expense / Inventory / Tax Receivable adjustment
```

### AP accounting handoff packet

```text
APAccountingHandoffPacket:
  bill_id
  supplier_id
  bill_status
  account_mapping_refs
  tax_treatment_ref
  amount_net
  tax_amount
  amount_gross
  currency
  accounting_period_id
  source_document_ref
  approval_refs
  payment_refs
  audit_ref
```

Accounting owns posting.

AP owns bill truth.

## 22. Tax / GST / VAT Claimability

AP must route tax treatment to Tax/Compliance.

AP captures:

```text
supplier tax registration
tax invoice validity
tax code candidate
tax amount
claimable percentage
non-claimable tax
country/region rule
business/private use split if applicable
reverse charge if applicable
withholding if applicable
```

Tax owner decides final treatment.

Selene may say:

```text
This invoice appears to include GST, but the supplier tax number is missing. I need tax review before it can be claimed.
```

GPT-5.5 may explain tax concepts.

It must not invent tax law.

## 23. Supplier Credit Notes and Adjustments

AP must support supplier credits.

Credit note reasons:

```text
returned goods
overbilling
price correction
tax correction
rebate
supplier refund
cancelled service
duplicate invoice correction
```

Credit note lifecycle:

```text
credit note received
-> supplier matched
-> original bill matched
-> tax treatment checked
-> AP balance adjusted
-> accounting entry prepared
-> payment schedule updated
-> audit
```

If supplier already paid:

```text
credit may become:
- supplier receivable
- future bill offset
- refund request
```

## 24. Supplier Disputes

If bill is wrong, Selene opens a dispute.

Dispute reasons:

```text
wrong amount
wrong tax
duplicate invoice
goods not received
service not completed
contractor hours mismatch
PO variance
damaged goods
wrong supplier bank details
price dispute
```

Dispute states:

```text
Opened
EvidenceCollecting
SupplierContacted
InternalReview
AwaitingSupplierResponse
Resolved
Rejected
CreditRequested
Cancelled
Closed
```

Selene may draft supplier message through PH1.WRITE.

Delivery uses BCAST/DELIVERY or secure supplier connector.

AP owns dispute state.

## 25. Supplier Statements

Suppliers may send statements.

Selene should reconcile supplier statement to AP.

```text
SupplierStatementPacket:
  statement_id
  supplier_id
  statement_date
  listed_invoices
  listed_payments
  supplier_balance
  company_ap_balance
  differences
  reconciliation_status
```

If mismatch:

```text
Selene identifies missing invoice, unallocated payment, duplicate, or supplier-side error.
```

## 26. Selene-to-Selene AP/AR Communication

If supplier also uses Selene, invoice delivery can happen system-to-system.

Flow:

```text
Supplier Selene sends invoice packet.
Buyer Selene receives as AP bill candidate.
Buyer Selene validates supplier, PO, tax, amount.
Buyer Selene routes approval.
Buyer Selene sends payment status back.
Supplier Selene updates AR.
Bank confirmation closes both sides where integrated.
```

Packet:

```text
SeleneToSeleneBillPacket:
  sender_company_id
  receiver_company_id
  invoice_id
  supplier_id
  buyer_id
  invoice_lines
  tax_details
  payment_terms
  due_date
  payment_instructions
  source_evidence_refs
  signature_or_auth_proof
  delivery_status
```

No email attachment if both systems support secure Selene-to-Selene delivery.

Delivery protocol must still be governed.

## 27. AP Reminders

PH1.REM handles timing.

BCAST/DELIVERY handles delivery.

AP source truth drives reminders.

Reminder types:

```text
bill_due_soon
bill_due_today
approval_pending
payment_failed
supplier_dispute_follow_up
contractor_overrun_approval
lease_payment_due
installment_due
supplier_statement_review
cashflow_review_needed
```

Selene should not remind randomly. She should remind because AP evidence says action is due.

## 28. Personal / Employee Spend Recovery Connected To AP

Employee or executive card personal spend may create a receivable or deduction, not an AP bill.

However AP may interact if:

```text
employee purchased supplier item
expense claim reimbursement required
card transaction belongs to supplier bill
personal spend requires company recovery
```

Recovery options belong mainly to future Credit Cards + Employee Spend + Reimbursements and Payroll/Accounting:

```text
payroll deduction where lawful and approved
employee repayment request
director/shareholder loan account
expense rejection
reimbursement reversal
```

AP should not pay personal charges as supplier obligations unless policy says so.

## 29. Employee Advances Connected To AP

Salary advances are primarily Payroll/Finance/Accounting.

AP may interact if payment is made outside payroll rails.

Rule:

```text
Employee advances must be tracked as recoverable amounts.

Repayment schedule must feed Payroll or Accounting.

No silent deduction without policy and employee-visible explanation.
```

Future Credit Cards + Employee Spend + Reimbursements and Payroll documents expand this.

## 30. Security and Privacy

AP data can be highly sensitive.

Sensitive fields:

```text
supplier bank details
contractor bank details
invoice amounts
payment approvals
tax IDs
supplier contracts
bank provider refs
payment confirmation refs
board approval notes
cashflow impact
```

Access controls required:

```text
view supplier bill
edit supplier bill
approve bill
approve payment
view supplier bank
change supplier bank
send payment
override cash reserve
export AP report
view supplier statement
view contractor invoices
```

Step-up required for:

```text
new supplier bank details
supplier bank change
high-value payment approval
urgent payment override
cash reserve breach override
payment cancellation
payment reversal
```

## 31. PH1.D / GPT-5.5 Role

Allowed:

```text
extract invoice fields
summarize supplier bill
suggest account category
explain PO variance
draft supplier dispute message
explain cashflow payment delay
draft approval summary
translate supplier message
```

Forbidden:

```text
approve bill
approve payment
execute payment
validate final supplier bank truth
invent tax code
override cash reserve
change payment terms
post journal
```

## 32. PH1.WRITE Wording

PH1.WRITE owns human wording.

### Approval request

```text
Supplier invoice INV-884 from North Forklift Repairs is ready for approval. The amount is AUD 6,200, which is AUD 420 above the purchase order. Approval is required before payment can be scheduled.
```

### Cashflow issue

```text
This payment would reduce cash below the current reserve rule. I can delay it if terms allow, prepare a partial payment, or request emergency approval.
```

### Supplier dispute

```text
We're reviewing invoice INV-884 because the amount is higher than the approved purchase order. We'll come back to you once the review is complete.
```

## 33. Audit Requirements

Audit must record:

```text
audit_event_id
actor_id
action
bill_id
supplier_id
payment_id
old_value_ref
new_value_ref
source_document_ref
purchase_order_ref
goods_received_ref
contract_ref
tax_rule_ref
budget_check_ref
cashflow_check_ref
approval_refs
step_up_refs
payment_provider_ref
bank_confirmation_ref
journal_ref
timestamp
company_id
legal_entity_id
country
currency
reason_code
```

No raw bank details in audit logs.

No hidden changes.

No supplier bank changes without trace.

## 34. Failure Branches

### Duplicate invoice

```text
Hold invoice.
Show suspected duplicate.
Require review.
No payment.
```

### Supplier bank changed

```text
Hold payment.
Require supplier bank verification and approval.
Step-up required for approver.
```

### Cash reserve breach

```text
Block normal payment.
Offer delay, partial payment, or emergency override.
```

### Approval timeout

```text
Reminder via PH1.REM.
Delivery via BCAST/DELIVERY.
Escalate if policy says.
No payment until approval.
```

### Payment provider failure

```text
Payment status failed.
Bill remains unpaid.
Banking/Payment creates failure proof.
AP notifies responsible owner.
```

### Contractor overrun

```text
Invoice exceeds approved hours/milestone.
Hold bill.
Route overrun approval.
```

### Closed accounting period

```text
Bill date falls in closed period.
Accounting period rule decides whether accrual/current-period posting/correction is required.
```

### Tax code missing

```text
Bill cannot be claimed/post-ready.
Route to Tax/Accounting.
```

## 35. Required Logical Packets

Future logical packets:

```text
SupplierProfilePacket
SupplierBankDetailPacket
SupplierBillIntakePacket
SupplierBillPacket
SupplierBillValidationPacket
DuplicateInvoiceCheckPacket
PurchaseOrderMatchPacket
GoodsReceiptMatchPacket
ContractorInvoiceMatchPacket
APApprovalRequestPacket
APApprovalDecisionPacket
PaymentTermsPacket
APScheduledPaymentPacket
APCashflowCheckRequestPacket
APPaymentPriorityPacket
APPaymentInstructionRequestPacket
APPaymentStatusPacket
APAccountingHandoffPacket
SupplierCreditNotePacket
SupplierDisputePacket
SupplierStatementPacket
SeleneToSeleneBillPacket
APReminderRequestPacket
APAuditEvidencePacket
```

Codex must later map these to repo equivalents.

Do not claim they currently exist.

## 36. Example End-To-End Flow — Supplier Bill

Supplier sends invoice for forklift repair.

```text
Invoice received.
Selene extracts supplier, amount, tax, due date.
Duplicate check passes.
Supplier exists and bank details approved.
PO match shows invoice is $420 above approved amount.
Selene creates variance exception.
Approval request sent to Operations Manager + Finance.
Approval passes.
Cashflow check passes.
Bill approved.
AP schedules payment for due date.
Payment instruction sent to Banking.
Bank confirms payment.
Accounting posts AP and bank entries.
Bank reconciliation matches payment.
Bill archived as paid.
```

Selene says:

```text
The forklift repair invoice is approved and scheduled for payment on its due date. It was above the purchase order by AUD 420, but Finance approved the variance. I'll reconcile it once the bank confirms payment.
```

## 37. Example End-To-End Flow — Lease Payment

Lease payment due monthly.

```text
Lease schedule active.
Next payment due in 5 days.
Selene checks cash reserve.
Cashflow is safe.
No additional approval required under policy.
Payment instruction prepared.
Bank payment executes on due date.
Accounting posts lease treatment.
Reconciliation confirms.
Next due date calculated.
```

Selene says:

```text
The monthly lease payment is scheduled and cashflow is safe. I'll process it on the due date and reconcile it after bank confirmation.
```

## 38. Example End-To-End Flow — Cash Reserve Breach

Supplier payment due tomorrow.

```text
Payment amount: AUD 450,000.
Reserve cash rule: minimum AUD 12,000,000.
Forecast cash after payment: AUD 11,840,000.
Reserve breach detected.
Normal payment blocked.
Emergency override required.
Selene routes approval to CFO + CEO.
```

Selene says:

```text
This payment would take cash below the AUD 12 million reserve. I can request emergency approval, propose a partial payment, or delay it if supplier terms allow.
```

## 39. What Must Not Happen

```text
no AP payment without source evidence
no duplicate invoice paid
no supplier bank change without verification and approval
no contractor invoice paid as employee payroll by default
no bill approved without required PO/contract/goods/service evidence
no cash reserve breach without emergency approval
no high-value payment without multi-authority approval where policy requires
no bank API call from AP alone
no AP journal posting without Accounting owner
no GST/VAT/tax claim invented by GPT-5.5
no supplier reminder or dispute message bypassing BCAST/DELIVERY where delivery is required
no raw supplier bank details in audit
no old AP records erased
no implementation from this document alone
```

## 40. Future Simulation Targets

```text
SIM_AP_001_supplier_bill_intake_and_duplicate_detection
SIM_AP_002_supplier_bill_po_variance_approval
SIM_AP_003_contractor_invoice_overrun_hold
SIM_AP_004_monthly_lease_payment_schedule
SIM_AP_005_cash_reserve_payment_block
SIM_AP_006_dual_approval_high_value_payment
SIM_AP_007_supplier_bank_change_payment_hold
SIM_AP_008_payment_provider_failure
SIM_AP_009_supplier_credit_note_application
SIM_AP_010_selene_to_selene_bill_delivery
SIM_AP_011_30_60_90_ap_aging_and_due_payment_schedule
```

## 41. Final Architecture Sentence

Selene Accounts Payable + Supplier Bills + Installments + Scheduled Payments is the governed outgoing-money engine: it receives and validates supplier bills, contractor invoices, purchase-order matches, goods/service evidence, tax treatment, installment schedules, lease payments, payment terms, cashflow checks, approval gates, payment instructions, bank confirmations, accounting handoffs, supplier disputes, credit notes, reminders, and audit so Selene can pay what the company owes on time, safely, automatically, and only through the correct authority, cashflow, banking, accounting, tax, and evidence owners.

## Related Addendum

Critical payment timing, no-late loan/lease/tax/payment rules, goods receiving, inspection/acceptance, supplier resolution, and 4-way purchase-to-pay matching are defined in SELENE_ACCOUNTS_PAYABLE_CRITICAL_PAYMENT_TIMING_GOODS_RECEIVING_SUPPLIER_RESOLUTION_ADDENDUM.md and must be read with this document.
