# Global Document 74 — Selene Supplier Payment + Banking Execution Handoff Engine

```text id="doc74_status"
DOCUMENT TYPE:
GLOBAL MASTER ARCHITECTURE DESIGN

GLOBAL DOCUMENT NUMBER:
74

ENGINE:
PH1.SUPPLIER_PAYMENT / PH1.BANKING_HANDOFF / PH1.PAYMENT_EXECUTION_CONTROL

FULL NAME:
Selene Supplier Payment, Banking Execution Handoff, Payment Scheduling, Batch Control, Rail Selection, Remittance, Payment Confirmation, Failure, Recall, and Cashflow-Safe Supplier Settlement Engine

STATUS:
FUTURE_CANONICAL_ARCHITECTURE
NOT_RUNTIME_IMPLEMENTATION
PENDING_REPO_TRUTH_ACTIVATION
PENDING_SIMULATION_MAPPING
CODEX_READY_MASTER_DESIGN
```

---

## 1. Purpose

Selene Supplier Payment + Banking Execution Handoff Engine owns the protected path from **AP-approved payable amount** to **bank/provider-submitted payment instruction, remittance, settlement confirmation, failure handling, and accounting proof**.

It answers:

```text id="payment_questions"
Which supplier invoices are payment-ready?
Which amounts are clean payable?
Which amounts remain held?
When should the supplier be paid?
Should payment be early, on due date, urgent, partial, split, delayed, or blocked?
Can cashflow support payment?
Is the supplier bank account verified and safe?
Is there a recent supplier bank change?
Is the supplier restricted, blocked, or under dispute?
Which payment rail/provider should be used?
Should this payment be batched?
Does authority permit submission?
Was the payment submitted?
Did the bank/provider accept it?
Did the payment settle?
Did the payment fail?
Was remittance sent?
Did AP, BankRec, Cashflow, and Accounting receive proof?
```

AP says:

```text id="ap_says"
This amount is payable.
This amount is held.
This invoice is ready for payment scheduling.
```

Supplier Payment says:

```text id="payment_says"
This payable amount can be safely scheduled, batched, submitted, confirmed, remitted, failed, retried, recalled, or closed.
```

Bank/provider says:

```text id="bank_says"
The money instruction was accepted, rejected, settled, failed, reversed, or remains pending.
```

Supplier Payment is not AP.

Supplier Payment is not Accounting.

Supplier Payment is not BankRec.

Supplier Payment is the protected execution handoff layer that makes sure clean supplier money moves safely, traceably, and intelligently.

Invoices do not get to walk themselves into the bank account wearing a little PDF hat.

---

## 2. Why Supplier Payment Comes After AP

The chain is:

```text id="supplier_payment_chain"
Supplier
→ Procurement / Purchase Order
→ Receiving / Inspection
→ AP / Creditors
→ Supplier Payment / Banking Execution Handoff
→ BankRec / Treasury
→ Accounting
→ Supplier Statement Reconciliation
```

AP validates invoice truth.

Supplier Payment handles payment timing and banking handoff.

BankRec proves what cleared.

Accounting posts final books.

Supplier Reconciliation checks supplier statements later.

If AP is skipped, payment becomes uncontrolled.

If BankRec is skipped, payment becomes assumed.

If Accounting is skipped, payment becomes invisible.

Selene must make supplier payment a controlled chain, not a “send money and hope” ritual. Hope remains a poor payment rail.

---

## 3. Modern Payment Standards Readiness

Selene must be ready for modern payment messaging, structured remittance, bank/provider APIs, payment status updates, and local payment rails.

ISO 20022 is a multi-part international financial messaging standard that provides a common platform for developing financial messages, and Swift describes ISO 20022 as enabling richer, better-structured, more granular payment data end-to-end. ([ISO20022][1])

Swift’s ISO 20022 payments guidance covers message families for payment initiation, payment clearing/settlement, and cash management reporting, so Selene should be architected to handle payment initiation, payment status, and account/reporting messages as structured events rather than plain-text bank notes. ([Swift][2])

For local payment rails, Same Day ACH is described by Nacha as a faster ACH method that allows payments to be sent and received within hours on the same business day, so Selene must treat rail choice, cut-off times, payment speed, fee, and risk as decision inputs rather than assuming all bank transfers behave the same way. ([Nacha][3])

Translation:

```text id="modern_payment_translation"
Supplier payment is not just “pay invoice.”
It is structured payment data, payment rail selection, bank safety, remittance, status tracking, reconciliation, failure handling, recall handling, and audit.
```

Old system:

```text id="old_payment"
Click pay.
Hope.
```

Selene:

```text id="selene_payment"
Validate payable truth.
Check supplier bank trust.
Check cashflow.
Check authority.
Choose payment rail.
Submit instruction.
Capture provider response.
Send remittance.
Verify settlement.
Update AP, BankRec, Cashflow, Accounting, Supplier, and Audit.
```

Tiny difference. Huge difference.

---

## 4. Core Selene Law

```text id="supplier_payment_core_law"
No supplier payment instruction may be scheduled, batched, or submitted unless AP has confirmed payable truth, Supplier has confirmed supplier safety, Supplier Bank Trust has confirmed bank safety, Cashflow has confirmed timing safety, Authority has approved protected actions, and Audit can prove the decision.
```

Selene must reduce human work by:

```text id="supplier_payment_reduce_human_work"
building payment queues
prioritizing supplier payments
checking AP readiness
checking supplier bank safety
checking cashflow
checking early payment discounts
checking due dates
checking supplier criticality
creating payment batches
selecting payment rails
generating remittance advice
submitting allowed payment instructions
capturing provider responses
tracking settlement
detecting failures
isolating failed batch items
updating AP
updating BankRec
updating Cashflow
preparing Accounting handoff
responding to supplier payment queries
```

Humans should only handle:

```text id="supplier_payment_human_review"
large payments
new supplier first payment
recent bank changes
fraud signals
cashflow overrides
payment despite disputes
manual urgent payments
recalls/reversals
settlements
restricted supplier payments
authority-protected exceptions
```

Selene should not ask humans to approve every clean $83 routine supplier payment like a nervous vending machine with a CFO badge.

---

## 5. Engine Boundary

### 5.1 PH1.SUPPLIER_PAYMENT owns

```text id="payment_owns"
payment readiness intake from AP
supplier payment queue
payment priority classification
payment schedule recommendation
payment schedule creation
cashflow-safe timing coordination
supplier bank safety check request
payment authority check request
payment rail/provider selection
payment batch creation
payment instruction preparation
payment file/API handoff
provider/bank submission response capture
payment settlement tracking
payment failure handling
partial batch failure handling
safe retry handling
payment cancellation/recall request workflow
remittance advice generation
supplier payment status response
AP payment status update
BankRec payment confirmation handoff
Accounting payment evidence handoff
Cashflow actual outflow feedback
payment audit evidence
```

### 5.2 PH1.SUPPLIER_PAYMENT does not own

```text id="payment_not_own"
supplier invoice validation
supplier credit note matching
goods receiving truth
supplier qualification truth
supplier bank change approval
cashflow policy ownership
ledger posting
tax treatment
bank account custody
supplier statement reconciliation
```

### 5.3 Correct owner split

```text id="payment_owner_split"
PH1.CREDITORS / AP = invoice payable truth and payment readiness.
PH1.SUPPLIER = supplier status, risk, criticality, restrictions.
PH1.SUPPLIER.BANK_TRUST = bank safety and bank-change protection.
PH1.CASHFLOW = liquidity, timing, payment prioritization policy.
PH1.ACCESS / AUTHORITY = protected payment approval.
PH1.SUPPLIER_PAYMENT = queue, schedule, batch, rail, submit instruction, remittance, status, failure, handoff.
PH1.BANKING / PAYMENT_PROVIDER = actual banking/payment provider acceptance and settlement.
PH1.BANKREC / TREASURY = bank proof and reconciliation.
PH1.ACCOUNTING = ledger posting.
PH1.AUDIT = evidence.
```

Supplier Payment owns the **execution handoff**.

The bank/provider owns actual money movement.

Accounting owns final books.

AP owns invoice truth.

This separation exists because giving one engine all of those powers is how you build a financial octopus with root access. Absolutely not.

---

## 6. Supplier Payment Master Record

Every supplier payment has a controlled record.

```text id="supplier_payment_record"
supplier_payment_id
legal_entity_id
supplier_id
supplier_status_snapshot
supplier_bank_safety_snapshot
AP_readiness_refs
invoice_refs
credit_note_refs
held_amount_refs
clean_payable_amount
payment_amount
currency
payment_priority
payment_reason
payment_terms
due_date
scheduled_payment_date
early_payment_discount_ref
cashflow_decision_ref
authority_ref
payment_rail
payment_provider
source_bank_account_ref
supplier_bank_ref
payment_batch_id
payment_instruction_ref
provider_submission_ref
provider_status
settlement_status
bank_confirmation_ref
remittance_ref
failure_ref
recall_ref
accounting_handoff_ref
audit_ref
```

Payment statuses:

```text id="supplier_payment_statuses"
Draft
PaymentReady
CashflowChecking
BankSafetyChecking
AuthorityChecking
Scheduled
BatchPending
InstructionPrepared
SubmittedToProvider
ProviderAccepted
ProviderRejected
BankProcessing
PendingSettlement
Confirmed
Failed
PartiallyFailed
Cancelled
RecallRequested
Reversed
APUpdated
BankRecUpdated
AccountingHandoffReady
Closed
Archived
```

Payment must be explainable at every stage.

If a supplier asks “where is my money?” Selene should not reply with “processing.” That word means nothing and has hurt society enough.

---

## 7. Payment Readiness Intake From AP

AP sends:

```text id="payment_readiness_packet"
PaymentReadinessPacket:
- supplier_id
- invoice_refs
- clean_payable_amount
- held_amount
- disputed_amount
- credit_notes_applied
- credit_notes_pending
- replacement_pending
- refund_pending
- due_date
- payment_terms
- early_payment_discount
- AP_readiness_status
- supplier_status_snapshot
- bank_safety_snapshot
- tax_status
- accounting_handoff_status
- audit_ref
```

Supplier Payment accepts only:

```text id="accepted_ap_statuses"
ReadyForPaymentScheduling
PartiallyReady
```

Supplier Payment rejects/holds:

```text id="blocked_ap_statuses"
BlockedSupplierRisk
BlockedBankRisk
BlockedDuplicate
BlockedNoPO
BlockedReceivingMismatch
BlockedInspectionPending
BlockedCreditPending
BlockedTaxReview
BlockedAuthority
```

Selene says:

> “AP marked $1,800 ready for payment scheduling. $200 remains on hold pending supplier credit.”

Payment Engine must respect partial readiness.

A clean amount should not be held hostage by a disputed amount unless policy requires it.

---

## 8. Payment Queue

Supplier Payment maintains a payment queue.

Queue fields:

```text id="payment_queue_fields"
queue_id
payment_id
supplier_id
invoice_refs
amount
currency
due_date
priority
cashflow_status
bank_safety_status
authority_status
payment_rail_candidate
scheduled_date
hold_reason
status
```

Queue categories:

```text id="payment_queue_categories"
ReadyToSchedule
Scheduled
DueSoon
DueToday
Overdue
CashflowWatch
BankSafetyHold
AuthorityRequired
SupplierDisputeHold
Rejected
Closed
```

Selene says:

> “There are 42 ready supplier payments. 36 are routine, 4 require cashflow timing review, and 2 are blocked by supplier bank safety.”

That is a useful payment run. Not “select invoices and pray.”

---

## 9. Payment Priority

Selene must classify outgoing supplier payments.

Payment priority categories:

```text id="payment_priority_categories"
MustPay
CriticalSupplier
StrategicSupplier
DiscountOpportunity
NormalDue
Flexible
DisputedHold
BankRiskHold
DoNotPay
```

### MustPay

```text id="must_pay"
statutory supplier obligation if applicable
court/settlement supplier payment
critical insurance premium
rent/lease if supplier-side and protected
contract default risk
```

### CriticalSupplier

```text id="critical_supplier"
supplier needed for production
supplier needed for customer delivery
supplier needed for safety/operations
supplier where non-payment creates supply stoppage
```

### DiscountOpportunity

```text id="discount_opportunity"
supplier offers early payment discount
cashflow can support
discount benefit outweighs cash cost/risk
invoice clean
bank safe
```

### DoNotPay

```text id="do_not_pay"
duplicate invoice
supplier blocked
bank unsafe
disputed goods
missing receiving proof
credit note pending for full amount
fraud signal
```

Selene says:

> “Pay the packaging supplier and courier today. Delay office stationery until due date. Do not pay Supplier ABC until bank verification completes.”

Payment priority is not “oldest invoice first.” That is calendar-based surrender.

---

## 10. Cashflow-Safe Payment Scheduling

Supplier Payment asks Cashflow before scheduling meaningful outflows.

Cashflow checks:

```text id="payment_cashflow_checks"
cleared cash
cash forecast
payroll buffer
tax buffer
rent/loan obligations
critical supplier needs
expected customer receipts
payment terms
discount opportunity
cash risk mode
minimum cash buffer
```

Cashflow outputs:

```text id="cashflow_decision_outputs"
pay now
pay on due date
pay early for discount
pay partial
split payment
delay within terms
negotiate extension
block due to cash risk
escalate for authority
```

Selene says:

> “Cashflow is yellow. I’ll pay critical suppliers and schedule flexible suppliers on their due dates.”

If discount:

> “Supplier offers a $200 discount for payment 20 days early. Cashflow remains above buffer, so I recommend taking it.”

If not safe:

> “The discount is attractive, but early payment would reduce payroll buffer. I recommend paying on due date.”

Selene is not cheap.

Selene is cash-aware.

If you can’t tell the difference, congratulations, you’ve discovered why many businesses run out of cash while profitable.

---

## 11. Supplier Bank Safety Check

Before payment scheduling/submission, Supplier Payment asks Supplier Bank Trust.

Bank safety statuses:

```text id="bank_safety_statuses"
Verified
PendingVerification
ChangeRequested
ChangeUnderReview
RecentlyChanged
Suspended
Rejected
Unknown
PaymentHold
```

Rules:

```text id="bank_safety_rules"
Verified = eligible under policy
RecentlyChanged = heightened review / possible hold
PendingVerification = hold
ChangeRequested = hold
ChangeUnderReview = hold
Suspended = block
Rejected = block
Unknown = block unless protected emergency authority
```

Selene says:

> “This invoice is clean, but supplier bank details changed recently. I removed it from the payment batch until verification completes.”

This is where money survives.

Urgent invoice + new bank details = fraud’s favourite little duet.

---

## 12. Authority Check

Supplier Payment asks Access / Authority for protected actions.

Authority triggers:

```text id="payment_authority_triggers"
payment above threshold
new supplier first payment
recent supplier bank change
manual urgent payment
payment despite cashflow warning
payment despite AP hold
restricted supplier payment
cross-border high-risk payment
foreign currency exposure
payment recall/reversal
settlement payment
board threshold
dual approval requirement
```

Authority outputs:

```text id="authority_outputs"
auto-allowed under policy
single approval required
dual approval required
board/executive approval required
blocked
```

Selene says:

> “This payment exceeds the automatic limit and requires CFO approval before submission.”

Payment approval is not AP approval.

AP says invoice is valid.

Authority says someone is allowed to send the money.

Different doors. Different keys. Stop giving one keyring to everything.

---

## 13. Payment Rail / Provider Selection

Selene chooses payment rail based on safety, speed, fee, currency, traceability, and supplier requirements.

Supported rail categories:

```text id="payment_rail_categories"
domestic bank transfer
ACH / local clearing rail
Same Day ACH where applicable
SEPA / regional transfer
wire transfer
real-time payment rail where configured
card payout
wallet payout
supplier portal payment
payment provider payout
escrow release
open banking payment initiation
manual bank file upload
ISO 20022 payment instruction where supported
```

Rail selection factors:

```text id="rail_selection_factors"
amount
currency
country
supplier bank country
urgency
fee
settlement speed
cut-off time
traceability
remittance data richness
fraud risk
provider reliability
cashflow timing
authority level
payment batch compatibility
```

Selene says:

> “Standard domestic transfer will meet the due date with lower cost. Urgent wire is unnecessary.”

Or:

> “Same-day rail is justified because non-payment may stop tomorrow’s production.”

Supplier Payment should not choose the fastest rail just because it exists. Fast mistakes are still mistakes, just with less recovery time.

---

## 14. Payment Batch Management

Selene groups payments into batches when appropriate.

Batch types:

```text id="payment_batch_types"
daily supplier batch
weekly supplier batch
urgent supplier batch
critical supplier batch
foreign currency batch
high-value batch
tax/government supplier batch
manual bank file batch
provider API batch
same-day batch
```

Batch record:

```text id="payment_batch_record"
payment_batch_id
legal_entity_id
source_bank_account_ref
payment_date
currency
payment_provider
payment_rail
payment_count
supplier_count
total_amount
risk_summary
cashflow_status
authority_status
bank_safety_status
submission_status
provider_response_ref
audit_ref
```

Batch states:

```text id="payment_batch_states"
Draft
PolicyValidated
CashflowChecked
BankSafetyChecked
AuthorityChecked
ReadyToSubmit
Submitted
ProviderAccepted
PartiallyAccepted
Rejected
Processing
Confirmed
PartiallyFailed
Failed
Closed
Archived
```

Selene says:

> “I created today’s supplier payment batch. 38 payments are clean. Two were excluded due to bank-safety hold.”

Good. Batch management should isolate problems, not throw the whole batch into a volcano because one supplier changed banks.

---

## 15. Payment Instruction

Payment instruction is what Selene submits or hands off to the bank/provider.

```text id="payment_instruction"
payment_instruction_id
supplier_payment_id
payment_batch_id
payer_legal_entity
source_bank_account_ref
supplier_id
supplier_bank_ref
beneficiary_name
amount
currency
payment_date
payment_rail
payment_provider
invoice_refs
remittance_refs
payment_reference
authority_ref
cashflow_ref
bank_safety_ref
idempotency_key
audit_ref
```

Mandatory protections:

```text id="instruction_protections"
idempotency key
duplicate payment check
supplier bank safety ref
AP readiness ref
authority ref where required
audit ref
```

No payment instruction without idempotency.

Duplicate payments are not “oops.” They are money cosplay.

---

## 16. Idempotency and Duplicate Payment Prevention

Selene must prevent duplicate payment submission.

Duplicate payment signals:

```text id="payment_duplicate_signals"
same supplier
same invoice
same amount
same payment date
same bank ref
same batch
same provider ref
same idempotency key
previous successful submission
payment retry without failure proof
```

Rules:

```text id="payment_duplicate_rules"
never submit same invoice/amount twice unless correction workflow exists
safe retry only after provider failure or unknown status protocol
unknown status requires reconciliation before resubmit
manual duplicate override requires protected authority
```

Selene says:

> “This payment may have already been submitted. I will not retry until provider status is confirmed.”

This prevents “click again” becoming “paid twice.” Very advanced. Somehow.

---

## 17. Provider Submission

Supplier Payment may hand off to:

```text id="provider_submission_targets"
bank API
payment provider API
bank file export
open banking initiation provider
supplier portal payment
manual bank upload queue
escrow platform
```

Submission response fields:

```text id="provider_response_fields"
provider_ref
submission_status
accepted/rejected
timestamp
reason_code
validation_errors
estimated_settlement_date
cut-off status
fees
FX quote/ref if applicable
audit_ref
```

Provider statuses:

```text id="provider_statuses"
NotSubmitted
Submitted
Accepted
Rejected
ValidationFailed
Processing
PendingSettlement
Confirmed
Failed
Unknown
```

Selene says:

> “The bank accepted the payment instruction. Settlement confirmation is pending.”

Important:

```text id="submission_not_settlement"
Provider accepted ≠ money settled.
Submitted ≠ paid.
Scheduled ≠ paid.
Payment-ready ≠ paid.
```

That sentence should be engraved into every finance system with a tiny hammer.

---

## 18. Settlement Tracking

Payment is not final until settlement/bank confirmation.

Confirmation sources:

```text id="settlement_sources"
bank/provider settlement confirmation
payment status message
bank statement
BankRec match
supplier acknowledgement
supplier Selene receipt
payment provider report
```

Settlement statuses:

```text id="settlement_statuses"
Pending
Confirmed
Failed
PartiallySettled
Returned
Reversed
Recalled
Unknown
```

Selene says:

> “Payment was submitted, but settlement is not confirmed. I’ll keep it pending and wait for BankRec/provider proof.”

No marking paid because something was clicked.

Clicks are not money movement.

Many systems have died on this hill. Let them.

---

## 19. Payment Failure Handling

Payment failures can happen.

Failure reasons:

```text id="payment_failure_reasons"
invalid bank details
bank account closed
beneficiary mismatch
insufficient funds
payment provider rejection
cut-off missed
compliance/sanctions hold
currency issue
duplicate blocked
bank system outage
network failure
manual bank file error
authority expired
```

Selene actions:

```text id="payment_failure_actions"
isolate failed payment
keep AP open
update supplier payment status
notify Supplier Payment owner
notify AP
notify Cashflow
notify supplier if appropriate
request verified bank update if needed
retry only if safe
route fraud review if bank mismatch
prepare accounting handoff only for confirmed payments
```

Selene says:

> “Payment failed because the bank rejected the account reference. I’ve kept the invoice unpaid and requested supplier bank verification.”

Payment failure should not disappear into “pending.” Pending is where truth goes to nap.

---

## 20. Partial Batch Failure

If one item in a batch fails, Selene must not lose the whole batch.

Partial failure handling:

```text id="partial_batch_handling"
identify successful payment lines
identify failed payment lines
update AP per line
update BankRec expected confirmations
notify affected supplier(s)
keep failed invoices open
do not resubmit successful items
create failure report
```

Selene says:

> “37 payments were accepted. One supplier payment failed due to bank validation. I isolated it and kept that invoice open.”

That is batch intelligence. Not “batch failed, scream.”

---

## 21. Payment Recall, Cancellation, and Reversal

Selene must support post-submission recovery paths.

Recovery actions:

```text id="payment_recovery_actions"
cancel before cut-off
recall request
bank reversal request
supplier refund request
duplicate payment recovery
fraud recovery escalation
legal escalation
```

Recall/reversal fields:

```text id="recall_fields"
payment_id
provider_ref
bank_ref
reason
amount
currency
submission_date
recall_deadline
provider_capability
status
authority_ref
audit_ref
```

Recall states:

```text id="recall_states"
NotRequired
Requested
AcceptedByProvider
RejectedByProvider
PendingBankResponse
Recovered
NotRecovered
Escalated
Closed
```

Selene says:

> “The payment has already been accepted by the bank. I can request recall, but recovery is not guaranteed. I’ll also notify AP and Supplier.”

No false hope. Just controlled damage.

Finance is a warm bath of consequences.

---

## 22. Remittance Advice

Selene sends supplier remittance after payment submission/confirmation according to policy.

Remittance includes:

```text id="remittance_fields"
supplier_id
payment_date
payment_amount
currency
payment_reference
invoice_refs
credit_notes_applied
withheld/disputed amounts
expected settlement date
bank/provider reference where allowed
contact/query path
```

Remittance states:

```text id="remittance_states"
Draft
Generated
Sent
Delivered
Acknowledged
Failed
Resent
Archived
```

Selene-connected supplier:

```text id="selene_remittance"
Buyer Selene sends SupplierPaymentRemittancePacket.
Supplier Selene matches to supplier AR.
```

Non-Selene supplier:

```text id="non_selene_remittance"
email
portal
secure link
PDF remittance
```

Selene says:

> “I paid invoices INV-884 and INV-885 today. Credit note CN-221 was applied. The damaged-goods amount remains on hold.”

This reduces supplier emails, which are basically mosquitoes with invoice attachments.

---

## 23. Supplier Payment Status Query

Suppliers may ask:

```text id="supplier_payment_queries"
When will we be paid?
Has payment been sent?
Why was payment short?
Why is payment on hold?
Was credit note applied?
Did payment fail?
```

Selene may answer with permitted data.

Allowed:

```text id="supplier_status_allowed"
invoice received
invoice matched
scheduled payment date
payment submitted
payment confirmed
amount on hold
credit note pending
replacement pending
bank verification pending
```

Not allowed:

```text id="supplier_status_not_allowed"
internal cashflow weakness
other supplier payments
internal approval politics
full bank details
restricted management notes
fraud investigation details beyond policy
```

Example:

> “Invoice INV-884 is scheduled for Friday. The remaining amount is on hold pending credit note for damaged goods.”

Useful. Safe. Slightly cold. Perfect AP energy.

---

## 24. Early Payment, Urgent Payment, and Discount Offers

Supplier Payment evaluates supplier payment requests with AP, Cashflow, Supplier, and Bank Trust.

### 24.1 Early payment request

Checks:

```text id="early_payment_checks"
invoice clean
no AP hold
supplier approved
bank verified
cashflow safe
business reason
supplier criticality
policy allows
```

Outcome:

```text id="early_payment_outcomes"
pay early
pay partial
pay on due date
ask for discount
decline
route approval
```

### 24.2 Urgent payment request

Checks:

```text id="urgent_payment_checks"
urgency reason
supply continuity risk
invoice readiness
bank safety
cashflow impact
supplier urgency pattern
fraud signal
authority required
```

Selene says:

> “Supplier requested urgent payment, but invoice includes disputed damaged goods. I will not accelerate payment until AP hold is resolved.”

### 24.3 Early payment discount

Selene calculates:

```text id="discount_calculation"
discount amount
days paid early
cashflow impact
annualized benefit
supplier criticality
opportunity cost
bank safety
payroll/tax/rent buffer
```

Outcome:

```text id="discount_outcome"
take discount
decline discount
negotiate better discount
pay partially
route approval
```

Selene says:

> “The supplier offers a 2% early payment discount. Cashflow stays above buffer, so I recommend taking it.”

Or:

> “The discount is attractive, but paying early would reduce payroll buffer. I recommend declining.”

Supplier requests are not orders. They are inputs. Selene has standards now.

---

## 25. Cross-Border and FX Payments

Supplier Payment must support foreign currency and cross-border payment complexity.

Inputs:

```text id="fx_payment_inputs"
invoice currency
payment currency
supplier country
supplier bank country
FX quote
FX expiry
fees
withholding tax flag
treaty/tax review flag
payment rail
settlement time
compliance screening status
```

Checks:

```text id="fx_payment_checks"
bank safety
supplier identity
FX rate
fees
withholding review
tax optimization review if relevant
cashflow impact
authority threshold
sanctions/compliance provider status where configured
```

Outputs:

```text id="fx_payment_outputs"
pay in invoice currency
pay in local currency
hold for tax/withholding review
request supplier banking clarification
route authority
```

Selene says:

> “This cross-border payment may require withholding tax review before payment. I’ll hold payment readiness until Tax confirms.”

Paying cross-border without tax/bank checks is how small invoices become large letters.

---

## 26. Supplier Payment and Tax

Supplier Payment must check Tax flags before payment where required.

Tax-related payment blocks:

```text id="tax_payment_blocks"
withholding tax review required
tax invoice evidence missing where payment policy requires
supplier tax status invalid
cross-border payment tax review
gross-up clause review
tax authority payment classification
```

Payment may proceed if policy allows, but tax treatment must be recorded.

Selene says:

> “Payment can proceed, but Tax has flagged withholding review. I’ll route before scheduling.”

Supplier Payment does not decide tax law.

Document 15 and 16 do.

We separated them for a reason, after shouting about tax optimization for half a day. Let’s honour the shouting.

---

## 27. Supplier Payment and Accounting Handoff

Supplier Payment sends Accounting payment evidence.

```text id="accounting_payment_handoff"
supplier_payment_id
supplier_id
invoice_refs
amount_paid
currency
payment_date
settlement_date
bank_account_ref
provider_ref
payment_reference
fees
FX difference candidate
credit notes applied
AP hold remaining
status
audit_ref
```

Accounting posts:

```text id="accounting_posts_payment"
Dr Accounts Payable
Cr Bank
fees / FX where applicable
```

Only after proof.

Supplier Payment does not post ledger.

It prepares evidence.

Accounting posts.

Repeating this because accounting boundaries have a habit of wandering off like cats.

---

## 28. Supplier Payment and BankRec Handoff

BankRec receives:

```text id="bankrec_payment_handoff"
expected bank outflow
payment_id
payment_batch_id
supplier_id
amount
currency
expected settlement date
provider_ref
bank_ref if available
payment_rail
status
```

BankRec later confirms:

```text id="bankrec_confirms"
payment cleared
payment failed
payment returned
amount mismatch
fee/FX difference
duplicate outflow
unknown status
```

Supplier Payment updates status from BankRec proof.

Selene says:

> “BankRec confirmed payment cleared. I’ve updated AP and prepared Accounting handoff.”

Payment is only truly closed when bank truth agrees.

No arguing with the bank statement. It is rude, and usually right.

---

## 29. Supplier Payment and AP Update

Supplier Payment updates AP:

```text id="ap_payment_update"
payment submitted
payment accepted
payment failed
payment confirmed
partial payment
amount paid
payment date
settlement date
remaining balance
remittance sent
failure reason
```

AP updates:

```text id="ap_updates"
invoice paid
partially paid
payment failed
still open
held balance remains
supplier balance base
```

Selene says:

> “Invoice INV-884 is now paid. The separate damaged-goods hold remains open.”

Payment should close only the paid portion.

Precision. A shocking innovation.

---

## 30. Supplier Payment and Supplier Reconciliation

Supplier Statement Reconciliation later uses:

```text id="payment_to_recon"
payment refs
invoice refs
credit notes applied
remittance sent
settlement proof
supplier acknowledgment
unapplied payment warning
```

Document 75 owns reconciliation.

Supplier Payment provides payment proof.

If supplier statement says unpaid but BankRec confirms payment, Reconciliation uses payment evidence.

Selene says:

> “Supplier statement does not show our payment. Reconciliation can send remittance and bank proof.”

Payment proof is ammunition for statement disputes. Harmless phrase. Very useful.

---

## 31. Supplier Payment and Cashflow Feedback

After payment, Cashflow receives actual outflow.

```text id="payment_cashflow_feedback"
actual payment date
actual amount
actual currency
fees
FX difference
failed payment reversal
cash buffer after payment
forecast variance
```

Cashflow learns:

```text id="cashflow_learning"
payment timing accuracy
supplier payment flexibility
discount value
late payment risk
critical supplier cash behavior
```

Selene says:

> “Actual supplier payments were $4,200 lower than forecast because two invoices remained on hold.”

Cashflow likes truth. It gets cranky when AP lies by accident.

---

## 32. Payment Security and Audit

Every supplier payment needs audit evidence.

Audit fields:

```text id="payment_audit_fields"
AP readiness proof
supplier approval proof
supplier bank safety proof
cashflow decision proof
authority proof
payment instruction
idempotency key
provider submission response
settlement confirmation
remittance record
AP update
BankRec update
Accounting handoff
actor/system
timestamp
audit_ref
```

Selene must answer:

> “Why did we pay Supplier ABC $18,000 on Friday?”

with:

```text id="payment_audit_answer"
invoice matched
goods accepted
credit notes applied
bank verified
cashflow approved timing
authority approved if required
payment submitted
bank confirmed
accounting handoff ready
```

No “because Jane said so.” Jane has suffered enough as a control environment.

---

## 33. Automation and Exception-Only Review

Selene auto-handles:

```text id="payment_auto_handles"
payment queue creation
routine payment scheduling
cashflow check requests
bank safety check requests
supplier payment priority classification
routine batch creation
remittance generation
provider response capture
failed payment isolation
AP status updates
BankRec expected payment handoff
Accounting payment evidence handoff
supplier payment status responses
```

Selene escalates:

```text id="payment_escalates"
high-value payment
new supplier first payment
supplier bank recently changed
bank safety hold
payment despite AP dispute
cashflow override
manual urgent payment
cross-border withholding/tax issue
payment recall/reversal
supplier settlement
fraud signal
authority conflict
provider unknown status
```

Rule:

```text id="payment_exception_rule"
Routine clean payment = Selene schedules and submits under policy.
Risky payment = Selene holds and routes.
Protected payment = authority approves.
Everything = audited.
```

No approval circus.

No payment free-for-all.

Two bad extremes. Selene picks the middle path, like a responsible adult who somehow got trapped in finance software.

---

## 34. PH1.D / GPT-5.5 Role

GPT-5.5 may help:

```text id="gpt_payment_allowed"
explain payment schedule
summarize supplier payment batch
draft supplier remittance note
draft supplier payment status response
explain failed payment reason
summarize cashflow tradeoff
draft payment approval summary
translate provider messages
prepare payment run briefing
```

GPT-5.5 must not:

```text id="gpt_payment_forbidden"
approve payment
submit payment
change payment amount
change supplier bank details
override AP hold
override bank safety hold
override cashflow block
mark payment confirmed
invent provider proof
release recall
post ledger
```

GPT-5.5 may write:

> “Payment is delayed because supplier bank verification is pending.”

GPT-5.5 may not decide:

> “Verification seems fine, send the money.”

No, little language model. Back away from the bank account.

---

## 35. Human-Like Selene Interaction

### Clean payment

> “This invoice is fully matched, bank details are verified, and cashflow is green. I’ve scheduled payment for the due date.”

### Partial payment

> “I’ll pay the accepted portion now and keep the damaged-goods amount on hold pending credit note.”

### Early discount

> “Supplier offers 2% discount if paid today. Cashflow supports it, so I recommend taking it.”

### Bank hold

> “The supplier changed bank details recently. I removed them from the payment batch until verification completes.”

### Failed payment

> “Payment failed because the bank rejected the account reference. I’ve kept the invoice open and requested verified bank details.”

### Batch summary

> “Today’s batch has 38 supplier payments. Two were excluded: one bank hold and one AP dispute.”

Human-like, precise, calm, and slightly suspicious. Perfect finance energy.

---

## 36. State Machines

### Supplier Payment State

```text id="supplier_payment_state"
Draft
PaymentReady
CashflowChecking
BankSafetyChecking
AuthorityChecking
Scheduled
BatchPending
InstructionPrepared
SubmittedToProvider
ProviderAccepted
ProviderRejected
BankProcessing
PendingSettlement
Confirmed
Failed
PartiallyFailed
Cancelled
RecallRequested
Reversed
APUpdated
BankRecUpdated
AccountingHandoffReady
Closed
Archived
```

### Payment Batch State

```text id="payment_batch_state"
Draft
PolicyValidated
CashflowChecked
BankSafetyChecked
AuthorityChecked
ReadyToSubmit
Submitted
ProviderAccepted
PartiallyAccepted
Rejected
Processing
Confirmed
PartiallyFailed
Failed
Closed
Archived
```

### Provider Submission State

```text id="provider_submission_state"
NotSubmitted
Submitted
Accepted
Rejected
ValidationFailed
Processing
PendingSettlement
Confirmed
Failed
Unknown
Archived
```

### Payment Failure State

```text id="payment_failure_state"
Detected
Classified
SupplierNotified
APUpdated
RetryEligible
RetryBlocked
RetrySubmitted
Resolved
Escalated
Closed
Archived
```

### Remittance State

```text id="remittance_state"
Draft
Generated
Sent
Delivered
Acknowledged
Failed
Resent
Archived
```

### Recall / Reversal State

```text id="recall_reversal_state"
NotRequired
Requested
AcceptedByProvider
RejectedByProvider
PendingBankResponse
Recovered
NotRecovered
Escalated
Closed
Archived
```

---

## 37. Reason Codes

```text id="payment_reason_codes"
PAYMENT_READY_FROM_AP
PAYMENT_PARTIALLY_READY_FROM_AP
PAYMENT_CASHFLOW_CHECK_REQUIRED
PAYMENT_CASHFLOW_GREEN
PAYMENT_CASHFLOW_WARNING
PAYMENT_CASHFLOW_BLOCK
PAYMENT_BANK_SAFETY_CHECK_REQUIRED
SUPPLIER_BANK_VERIFIED
SUPPLIER_BANK_CHANGE_HOLD
SUPPLIER_BANK_SUSPENDED
PAYMENT_AUTHORITY_REQUIRED
PAYMENT_AUTHORITY_APPROVED
PAYMENT_SCHEDULED
PAYMENT_BATCH_CREATED
PAYMENT_BATCH_READY
PAYMENT_INSTRUCTION_PREPARED
PAYMENT_SUBMITTED_TO_PROVIDER
PROVIDER_ACCEPTED
PROVIDER_REJECTED
PAYMENT_PENDING_SETTLEMENT
PAYMENT_CONFIRMED
PAYMENT_FAILED
PAYMENT_PARTIALLY_FAILED
PAYMENT_DUPLICATE_RISK
PAYMENT_RETRY_BLOCKED_UNKNOWN_STATUS
EARLY_PAYMENT_DISCOUNT_RECOMMENDED
URGENT_PAYMENT_REQUEST_REVIEW
REMITTANCE_GENERATED
REMITTANCE_SENT
BANKREC_HANDOFF_READY
ACCOUNTING_HANDOFF_READY
PAYMENT_RECALL_REQUESTED
PAYMENT_REVERSED
```

---

## 38. Required Simulations

```text id="payment_simulations"
clean AP-ready invoice scheduled for payment
partial AP-ready invoice with AP hold
supplier bank verified payment scheduled
supplier bank change blocks payment
supplier early payment request evaluated
supplier urgent payment request evaluated
early payment discount accepted
early payment discount rejected due to cashflow
payment batch created
payment batch excludes bank-risk supplier
payment instruction submitted to provider
provider accepts payment
provider rejects payment
payment pending settlement
BankRec confirms payment cleared
payment failed invalid bank
partial batch failure isolated
duplicate payment attempt blocked
unknown provider status blocks retry
supplier remittance sent
supplier asks payment status
cross-border payment tax review hold
payment recall requested
payment recall fails
accounting payment handoff created
cashflow actual outflow updated
```

---

## 39. Integration Map

```text id="payment_integration_map"
PH1.SUPPLIER_PAYMENT / BANKING_HANDOFF
↔ PH1.CREDITORS / AP
↔ PH1.SUPPLIER
↔ PH1.SUPPLIER.BANK_TRUST
↔ PH1.CASHFLOW
↔ PH1.BANKING / PAYMENT_PROVIDER
↔ PH1.BANKREC / TREASURY
↔ PH1.ACCOUNTING
↔ PH1.TAX
↔ PH1.TAX.OPTIMIZE
↔ PH1.BUDGET
↔ PH1.CREDITORS.RECON
↔ PH1.LEGAL / CONTRACTS
↔ PH1.COMPLIANCE
↔ PH1.ACCESS / AUTHORITY
↔ PH1.AUDIT
↔ PH1.REM
↔ PH1.BCAST / DELIVERY
↔ PH1.WRITE
↔ PH1.D / GPT-5.5
```

---

## 40. Required Logical Packets

```text id="payment_packets"
PaymentReadinessPacket
SupplierPaymentPacket
SupplierPaymentQueuePacket
PaymentPriorityPacket
CashflowPaymentDecisionPacket
SupplierBankSafetyPacket
PaymentAuthorityPacket
PaymentSchedulePacket
PaymentBatchPacket
PaymentInstructionPacket
ProviderSubmissionPacket
ProviderResponsePacket
PaymentSettlementPacket
PaymentFailurePacket
PaymentRetryPacket
PaymentRecallPacket
PaymentReversalPacket
SupplierRemittancePacket
SupplierPaymentStatusQueryPacket
APPaymentStatusUpdatePacket
BankRecPaymentHandoffPacket
AccountingPaymentHandoffPacket
CashflowPaymentFeedbackPacket
PaymentAuditEvidencePacket
```

Logical only. Codex maps later. No packet structs yet, no matter how official the names look. Sit down, schema goblin.

---

## 41. What Codex Must Not Do

```text id="codex_no_payment"
Do not merge Supplier Payment with AP.
Do not let Supplier Payment validate invoices.
Do not let Supplier Payment own supplier bank changes.
Do not let Supplier Payment post ledger.
Do not let Supplier Payment replace BankRec.
Do not mark submitted as paid.
Do not mark provider accepted as settled.
Do not retry unknown-status payments without proof.
Do not bypass AP holds.
Do not bypass bank safety holds.
Do not bypass cashflow blocks.
Do not let GPT-5.5 approve or submit payments.
Do not create runtime banking code from this document.
Do not create packet structs.
Do not implement from this document alone.
```

---

## 42. Final Architecture Sentence

Selene Supplier Payment + Banking Execution Handoff Engine is the protected supplier settlement layer that receives AP payment readiness, checks supplier status, supplier bank safety, cashflow, payment priority, due dates, early-payment discounts, urgency, authority, payment rail, batch eligibility, duplicate risk, and tax/cross-border flags; prepares payment queues, schedules, batches, instructions, remittances, provider submissions, failure workflows, recalls, AP updates, BankRec handoffs, Accounting handoffs, and Cashflow feedback; and uses GPT-5.5 for human explanation while deterministic Selene policy, AP truth, bank safety, cashflow, authority, provider proof, BankRec confirmation, accounting, and audit protect money movement.

Simple version:

```text id="payment_simple"
AP says what is payable.
Supplier Payment checks cashflow.
Supplier Payment checks supplier bank safety.
Supplier Payment checks authority.
Selene schedules payment.
Selene batches payment.
Selene submits payment instruction.
Bank/provider accepts or rejects.
BankRec proves settlement.
AP updates invoice status.
Accounting gets payment proof.
Supplier gets remittance.
Humans approve only risky exceptions.
Everything is audited.
```

That is Global Document 74 — Supplier Payment + Banking Execution Handoff Engine. The invoice truth has now reached the money door, and Selene stands there with AP proof, bank-safety checks, cashflow sense, authority rules, duplicate protection, remittance discipline, and a firm refusal to let a supplier’s urgent email sprint straight into the bank account.

[1]: https://www.iso20022.org/iso-20022?utm_source=chatgpt.com "ISO 20022 | ISO20022"
[2]: https://www.swift.com/standards/iso-20022/iso-20022-financial-institutions-focus-payments-instructions?utm_source=chatgpt.com "ISO 20022 for Financial Institutions - Swift"
[3]: https://www.nacha.org/same-day-ach?utm_source=chatgpt.com "Same Day ACH"

---

## 43. 81E B2B Settlement Hold + Payout Handoff

Supplier Payment must respect 81E settlement holds, reserves, refund exposure, reversal exposure, clawback rules, customer benefit funding, provider payout amount, Channel Store commission obligation, Selene B2B fee allocation, and return courier/reverse logistics allocation before submitting or releasing supplier/provider payments.

Provider payout and Channel Store commission obligations must remain separate, auditable settlement concepts. Supplier Payment may execute only after AP readiness, bank safety, cashflow, authority, and 81E/Document 78 hold-release rules are satisfied.

---

## 44. 81F-81G Pricing Evidence + Settlement Handoff

Supplier Payment must respect 81G audit, approval, version, dispute, and evidence references for pricing-related payouts, reversals, provider settlements, Channel Store commissions, customer benefit funding, cashback/credit liabilities, and promotion-related settlement changes.

B2B settlement holds, promotion reversals, provider payouts, customer benefit funding, and commission clawbacks must not release without required 81E and 81G evidence, AP readiness, authority, bank safety, and cashflow clearance.

---

## 45. Commerce Stack 83 Settlement Hold + Reversal Handoff

Supplier Payment must not release provider payout, Channel Store commission, supplier payment, customer benefit funding, reserve release, or settlement hold where Document 83, 81E, 81G, AP, or authority state indicates refund, return, warranty, dispute, credit note, clawback, abuse, or provider responsibility is unresolved.

Payment release, reversal, recall, or provider payout adjustment must preserve the Document 83 dispute/return evidence and the 81G audit reference.
