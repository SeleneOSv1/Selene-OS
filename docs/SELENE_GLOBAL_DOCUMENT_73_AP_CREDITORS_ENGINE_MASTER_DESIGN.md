# Global Document 73 — Selene AP / Creditors Engine

```text id="doc73_status"
DOCUMENT TYPE:
GLOBAL MASTER ARCHITECTURE DESIGN

GLOBAL DOCUMENT NUMBER:
73

ENGINE:
PH1.CREDITORS / PH1.ACCOUNTS_PAYABLE / PH1.AP_MATCHING

FULL NAME:
Selene Accounts Payable, Creditors, Supplier Invoice, Credit Note, AP Hold, Payable Amount, Duplicate Detection, Supplier Balance Base, and Payment Readiness Engine

STATUS:
FUTURE_CANONICAL_ARCHITECTURE
NOT_RUNTIME_IMPLEMENTATION
PENDING_REPO_TRUTH_ACTIVATION
PENDING_SIMULATION_MAPPING
CODEX_READY_MASTER_DESIGN
```

---

## 1. Purpose

Selene AP / Creditors Engine owns the supplier invoice truth.

It answers:

```text id="ap_questions"
Did this supplier invoice come from a real supplier?
Is the supplier approved?
Is the invoice duplicate?
Does the invoice match a purchase order?
Did the goods or services actually arrive?
Were they accepted?
Were any goods short, damaged, rejected, quarantined, or disputed?
Is a credit note required?
Is a replacement pending?
Is a refund pending?
What amount is clean payable?
What amount must be held?
Is the supplier bank safe before payment readiness?
Is the invoice ready for payment scheduling?
What should Accounting receive as evidence?
```

AP is not the payment engine.

AP is not the bank.

AP is not the supplier statement reconciliation engine.

AP is not the ledger.

AP is the invoice truth gate.

A supplier invoice is a **claim**.

Selene AP decides whether that claim matches:

```text id="ap_claim_match"
approved supplier
approved purchase order
receiving proof
inspection proof
credit notes
supplier obligations
payment terms
tax evidence
duplicate checks
policy
audit
```

Old AP:

```text id="old_ap"
Invoice arrives.
Someone forwards it.
Someone asks if anyone ordered it.
Someone pays it because it looks official.
Supplier later owes a credit note.
Nobody remembers.
```

Selene AP:

```text id="selene_ap"
Invoice arrives.
Selene reads it.
Selene matches it.
Selene holds disputed value.
Selene requests credit/replacement/refund.
Selene marks clean payable amount.
Selene prepares payment readiness.
Selene sends accounting handoff.
```

That is AP with a spine. Not an inbox with anxiety.

---

## 2. Why AP Comes After Receiving

The chain must stay clean:

```text id="ap_after_receiving"
Procurement says what was ordered.
Receiving proves what arrived.
Inspection proves what was accepted.
Supplier Engine tracks supplier obligations and risk.
AP validates the supplier invoice against that proof.
Supplier Payment schedules and executes payment only after AP readiness.
Accounting posts final financial truth.
```

AP must not trust invoices alone.

AP must not add stock.

AP must not release payment.

AP must not close supplier obligations without proof.

AP’s job is:

```text id="ap_job"
Turn supplier invoice claims into clean payable, held, rejected, disputed, credited, or payment-ready amounts.
```

If AP receives an invoice for 100 units, but Receiving accepted 90, AP must treat 90 as clean and 10 as disputed until credit, replacement, refund, or approved exception resolves it.

A supplier invoice saying “100” is not stronger than a warehouse photo showing 10 broken units. Sorry, PDF.

---

## 3. Core Selene Law

```text id="ap_core_law"
No supplier invoice becomes clean payable unless it passes supplier, PO, receiving, inspection, duplicate, credit, tax, bank-safety, policy, and audit checks required for that invoice type.

Routine matched invoices are processed autonomously by Selene under policy.
Humans review only exceptions, disputes, high-risk suppliers, material variances, missing evidence, policy overrides, and protected approvals.
```

Selene must reduce human work by:

```text id="ap_reduce_human"
capturing invoices automatically
reading supplier invoice fields
matching invoices to suppliers
detecting duplicate invoices
matching invoices to POs
matching invoices to receiving proof
matching invoices to inspection proof
applying credit notes
tracking replacement/refund status
creating AP holds
calculating payable amount
checking supplier bank safety before payment readiness
checking payment terms
preparing payment readiness
preparing accounting handoff
drafting supplier queries
responding to supplier invoice status questions
```

Humans should not spend their day comparing three PDFs, one warehouse photo, a PO, and a vague email from Supplier ABC.

Selene should do that.

Humans can review the weird bits, because apparently humans still want to feel included.

---

## 4. Engine Boundary

### 4.1 PH1.CREDITORS owns

```text id="ap_owns"
supplier invoice intake
supplier invoice extraction
supplier invoice classification
supplier matching
invoice duplicate detection
invoice-to-PO matching
invoice-to-receiving matching
invoice-to-inspection matching
credit note matching
replacement tracking
refund tracking
AP hold creation
disputed amount calculation
clean payable amount calculation
payment terms capture
supplier balance base
invoice status lifecycle
payment readiness handoff
accounting evidence handoff
supplier invoice dispute workflow
tax evidence handoff
AP audit evidence
```

### 4.2 PH1.CREDITORS does not own

```text id="ap_not_own"
supplier qualification final truth
purchase order creation
goods receiving acceptance
inventory stock truth
supplier bank change approval
payment execution
bank transfer
ledger posting
supplier statement reconciliation final process
tax law final treatment
budget ownership
cashflow ownership
```

### 4.3 Correct owner split

```text id="ap_owner_split"
PH1.SUPPLIER = supplier identity, approval, risk, obligations, bank safety signal.
PH1.PROCUREMENT = purchase order and buying proof.
PH1.PROC.RECEIVE = receiving/inspection proof and accepted quantity.
PH1.CREDITORS / AP = invoice matching, payable amount, holds, payment readiness.
PH1.SUPPLIER_PAYMENT = payment scheduling and bank/provider handoff.
PH1.CREDITORS.RECON = supplier statement reconciliation and creditor reporting.
PH1.ACCOUNTING = ledger posting and AP control account.
PH1.CASHFLOW = liquidity and payment timing.
PH1.TAX = tax compliance and tax classification.
PH1.AUDIT = proof.
```

AP is the invoice judge.

Payment is the cashier.

Accounting is the bookkeeper of record.

Do not give one goblin all three hats.

---

## 5. Supplier Invoice Intake

Selene must capture invoices from many sources.

```text id="invoice_sources"
supplier email
supplier portal
Supplier Selene
PDF upload
photo/scanned invoice
paper invoice capture
EDI / e-invoice feed
B2B platform
marketplace supplier feed
contract/recurring bill schedule
payment provider supplier invoice feed
manual AP upload
```

Selene extracts:

```text id="invoice_extraction_fields"
invoice_id
supplier_id
supplier_name
supplier_tax_id
invoice_number
invoice_date
due_date
currency
payment_terms
PO_number
delivery_note_ref
receiving_ref
line_items
quantities
unit_prices
tax_amounts
total_amount
bank_details_ref
credit_note_refs
discount_terms
attachments
source_channel
audit_ref
```

Selene says:

> “I found invoice INV-884 from Supplier ABC for PO-771. I’ll match it to receiving and inspection.”

If supplier is unknown:

> “This invoice is from an unknown supplier. I will not process payment readiness until Supplier review is complete.”

Unknown supplier invoice is not AP. It is an invitation to fraud with a logo.

---

## 6. Supplier Invoice Master Record

Every invoice must have a controlled record.

```text id="supplier_invoice_record"
supplier_invoice_id
legal_entity_id
supplier_id
supplier_status_snapshot
supplier_bank_safety_snapshot
invoice_number
invoice_date
received_date
due_date
currency
invoice_total
tax_total
net_total
payment_terms
source_channel
PO_refs
receiving_refs
inspection_refs
line_items
credit_note_refs
replacement_refs
refund_refs
AP_hold_refs
duplicate_check_status
matching_status
tax_review_status
payment_readiness_status
accounting_handoff_status
invoice_status
audit_ref
```

Invoice statuses:

```text id="invoice_statuses"
Received
ExtractionPending
Extracted
SupplierMatching
SupplierMatched
DuplicateChecking
DuplicateSuspected
MatchingPending
Matched
PartiallyMatched
MismatchDetected
CreditRequired
ReplacementPending
RefundPending
PaymentHold
ExceptionReview
ReadyForPayment
PaymentRequested
Paid
PartiallyPaid
Rejected
Cancelled
Archived
```

AP must be able to explain every status in plain English.

If a supplier asks “why not paid?” Selene should not respond with “status 7B.” That is how software becomes furniture.

---

## 7. Invoice Line Matching

AP matches at invoice-line level, not just total.

Invoice line fields:

```text id="invoice_line_fields"
invoice_line_id
supplier_invoice_id
product_id
service_id
asset_id
supplier_sku
description
quantity_invoiced
unit_of_measure
unit_price
line_net_amount
tax_amount
line_total
PO_line_ref
receiving_line_ref
inspection_ref
matching_status
disputed_quantity
disputed_amount
credit_required
replacement_pending
refund_pending
audit_ref
```

Line statuses:

```text id="line_statuses"
Unmatched
Matched
PartiallyMatched
QuantityMismatch
PriceMismatch
TaxMismatch
NoPO
NoReceiving
InspectionPending
DamagedOrRejected
CreditRequired
Held
ApprovedPayable
Rejected
```

Example:

```text id="line_example"
Invoice line:
100 units @ $20

Receiving:
90 accepted
10 damaged

AP line:
90 payable
10 held / credit required
```

AP total matching without line matching is how bad invoices sneak in politely.

---

## 8. Duplicate Invoice Detection

Selene must detect duplicate invoices automatically.

Signals:

```text id="duplicate_signals"
same supplier + same invoice number
same supplier + same invoice amount + same date
same PO + same invoice amount
same attachment hash
same PDF visual similarity
same line items
same bank reference
same supplier statement claim already processed
credit note reversed and re-invoiced incorrectly
```

Duplicate outcomes:

```text id="duplicate_outcomes"
clear
possible duplicate
confirmed duplicate
supplier clarification required
blocked from payment readiness
```

Selene says:

> “This invoice appears to duplicate INV-884 from Supplier ABC. I’ve blocked payment readiness and linked both records.”

No one should pay twice because the supplier sent “final_invoice_v2_really.pdf.” That file name is not a control.

---

## 9. Matching Models

AP supports multiple matching models based on risk and transaction type.

### 9.1 Two-way match

```text id="two_way_match"
Purchase Order
+ Supplier Invoice
```

Used when receiving proof is not required or where service acceptance is controlled elsewhere.

### 9.2 Three-way match

```text id="three_way_match"
Purchase Order
+ Goods Receipt
+ Supplier Invoice
```

Used for standard goods.

### 9.3 Four-way match

```text id="four_way_match"
Purchase Order
+ Goods Receipt
+ Inspection / Accepted Quantity
+ Supplier Invoice
```

Used for physical goods with quality, condition, expiry, batch, serial, compliance, or acceptance requirements.

### 9.4 Contract/recurring match

```text id="contract_match"
Contract / recurring schedule
+ Service period / usage
+ Supplier Invoice
```

Used for rent, utilities, subscriptions, retainers, leases, and recurring service providers.

### 9.5 No-match / exception path

```text id="no_match_path"
Supplier Invoice
+ no approved buying/receiving/contract evidence
= payment blocked pending review
```

Selene says:

> “This invoice has no PO, no receiving event, and no recurring bill record. I will not mark it payment-ready.”

A supplier invoice without matching business proof is not an invoice to pay. It is a question.

---

## 10. Four-Way Match Logic

For goods requiring strong proof:

```text id="four_way_logic"
PO quantity and price
Receiving quantity delivered
Inspection accepted quantity
Invoice quantity and price
```

Clean payable amount:

```text id="clean_payable_formula"
accepted_quantity × approved_unit_price
```

Held amount:

```text id="held_amount_formula"
(invoice_quantity - accepted_quantity) × approved_unit_price
or actual disputed invoice amount where different
```

Example:

```text id="four_way_example"
PO: 100 units @ $20
Received: 100
Accepted: 90
Damaged: 10
Invoice: 100 units @ $20

Clean payable: $1,800
Held: $200
Supplier obligation: credit/replacement/refund for 10 units
```

Selene says:

> “Invoice includes 100 units, but only 90 were accepted. I’ve marked $1,800 clean payable and held $200 pending supplier credit or replacement.”

That is AP truth.

Everything else is paying for optimism.

---

## 11. AP Holds

AP hold means invoice amount is not clean payable yet.

Hold triggers:

```text id="ap_hold_triggers"
no matching supplier
supplier not approved
supplier blocked/restricted
supplier bank unsafe
no PO
PO mismatch
receiving missing
inspection pending
short delivery
damaged goods
wrong goods
rejected goods
quarantine
credit note pending
replacement pending
refund pending
duplicate invoice suspected
tax evidence missing
manual review required
```

Hold types:

```text id="ap_hold_types"
supplier hold
invoice hold
line-item hold
disputed amount hold
bank safety hold
tax evidence hold
duplicate hold
receiving hold
inspection hold
credit note hold
replacement hold
refund hold
```

Hold states:

```text id="ap_hold_states"
NoHold
HoldSuggested
HoldApplied
EvidenceRequested
AwaitingCreditNote
AwaitingReplacement
AwaitingRefund
AwaitingApproval
Released
Rejected
Closed
```

Selene says:

> “I’ve applied a line-item hold for the damaged goods. The rest of the invoice remains payment-ready.”

Good AP does not block clean amounts unnecessarily.

It holds only what is actually disputed.

Because vendors still need to be paid when they deserve it. Annoying, but true.

---

## 12. Credit Note Handling

AP must match supplier credit notes to the original issue.

Credit note triggers:

```text id="credit_note_triggers"
short delivery
damaged goods
wrong goods
rejected goods
overcharge
duplicate invoice
price correction
returned goods
service failure
supplier settlement
```

Credit note fields:

```text id="credit_note_fields"
credit_note_id
supplier_id
credit_note_number
credit_note_date
original_invoice_id
PO_ref
receiving_ref
supplier_obligation_ref
amount
tax_amount
currency
reason
matching_status
audit_ref
```

Credit note statuses:

```text id="credit_note_statuses"
Expected
Requested
Received
Matched
PartiallyMatched
Mismatch
Applied
Rejected
Closed
```

Selene says:

> “Credit note CN-221 matches the five damaged units from PO-771. I’ve applied it and released the AP hold.”

If credit note amount does not match:

> “Supplier credit note is $160, but expected credit is $200. I’ll keep the remaining $40 hold open.”

Supplier almost-right is still not right. Very sad. Still true.

---

## 13. Replacement Handling

Sometimes supplier resolves by sending replacement goods instead of credit.

AP tracks replacement status because payment may depend on accepted replacement.

Replacement states:

```text id="replacement_states"
NotRequired
Requested
SupplierAcknowledged
Shipped
Received
Accepted
Rejected
Closed
```

If replacement accepted:

```text id="replacement_accepted"
supplier obligation closes
AP hold may release if invoice now matches accepted goods
Inventory receives replacement through Receiving
Supplier score updated
```

If replacement rejected:

```text id="replacement_rejected"
AP hold remains
supplier obligation remains open
credit/refund may be requested
```

Selene says:

> “Replacement goods were received and accepted. The invoice now matches accepted quantity, so I can release the AP hold.”

Replacement is not resolved when shipped.

It is resolved when accepted.

Suppliers sometimes ship optimism too.

---

## 14. Refund Handling

Refunds happen when supplier must return money.

Refund triggers:

```text id="refund_triggers"
invoice already paid
supplier overpaid
credit note not suitable
order cancelled after payment
service not delivered
settlement requires cash refund
```

Refund status:

```text id="refund_statuses"
Expected
Requested
SupplierAcknowledged
PendingReceipt
Received
MatchedToBank
Applied
Overdue
Escalated
Closed
```

AP tracks refund receivable or supplier balance adjustment.

BankRec proves money arrived.

Selene says:

> “Supplier refund is expected because the invoice was already paid. I’ll keep it open until BankRec confirms receipt.”

Refund promised is not refund received. This lesson has hurt many companies and most optimism.

---

## 15. Supplier Invoice Dispute Workflow

AP dispute starts when invoice claim conflicts with proof.

Dispute reasons:

```text id="invoice_dispute_reasons"
invoice quantity exceeds accepted quantity
invoice price exceeds PO
invoice includes damaged goods
invoice includes rejected goods
invoice has no PO
invoice duplicates prior invoice
tax amount incorrect
supplier claims unpaid but already paid
supplier refuses credit note
supplier statement includes disputed invoice
```

Dispute states:

```text id="invoice_dispute_states"
Detected
EvidenceAttached
SupplierNotified
AwaitingSupplierResponse
SupplierResponded
AcceptedBySupplier
RejectedBySupplier
InternalReview
Escalated
Resolved
Closed
```

Selene may draft:

> “Our records show only 90 accepted units against PO-771. Please issue a credit note for the 10 damaged units. Evidence attached.”

GPT may draft. Selene facts must come from deterministic records.

No poetic dispute emails inventing quantities because the model felt persuasive.

---

## 16. Non-PO Invoices

Some invoices are valid without PO.

Examples:

```text id="non_po_examples"
rent
utilities
bank fees
insurance premiums
tax authority bills
approved subscriptions
loan fees
approved recurring service contracts
emergency purchases
professional retainers
```

Non-PO invoice requires one of:

```text id="non_po_evidence"
approved recurring bill record
contract
lease
utility account
tax obligation
emergency approval
service acceptance
authority exception
```

If no evidence:

```text id="non_po_block"
NO_PO_NO_APPROVED_RECORD
AP hold
Procurement/Finance review
payment readiness blocked
```

Selene says:

> “This invoice has no PO and no approved recurring record. I’ve blocked payment readiness pending review.”

Not every invoice needs a PO.

But every invoice needs a reason.

Subtle difference. Harder than it should be.

---

## 17. Recurring Bills and Periodic Supplier Charges

AP supports recurring supplier invoices.

Recurring types:

```text id="recurring_bill_types"
rent
utilities
insurance
software subscriptions
maintenance contracts
leases
retainers
telecom/internet
waste collection
security services
cleaning services
bank fees
loan fees
```

Recurring bill record:

```text id="recurring_bill_record"
recurring_bill_id
supplier_id
contract_ref
service_period
frequency
expected_amount
variance_tolerance
due_date_pattern
payment_terms
cost_center
tax_hint
approval_policy
renewal_date
status
audit_ref
```

Selene checks:

```text id="recurring_checks"
amount within tolerance
service period correct
duplicate period not billed
supplier approved
contract active
tax evidence present
budget/cashflow impact
```

Selene says:

> “This subscription invoice is 18% higher than the approved recurring amount. I’ve held the increase for review while leaving the normal amount payable.”

That is smarter than paying the surprise increase because the vendor called it “updated pricing.”

---

## 18. Supplier Bank Safety Before Payment Readiness

AP does not approve payment execution, but AP must check bank safety before marking payment-ready.

AP asks Supplier Bank Trust:

```text id="bank_safety_query"
bank verified?
bank recently changed?
bank change pending?
supplier payment hold?
bank fraud signal?
```

Payment readiness blocked if:

```text id="bank_safety_blocks"
bank not verified
bank change pending
supplier bank suspended
fraud review open
supplier payment hold active
```

Selene says:

> “Invoice is clean, but supplier bank details are under review. I will not mark it payment-ready.”

Clean invoice + unsafe bank = still not payable.

The invoice may be fine. The payment destination may be a trap. Both matter.

---

## 19. Payment Readiness

AP does not pay.

AP marks an invoice or amount as payment-ready.

Payment readiness requires:

```text id="payment_readiness_requires"
supplier approved
supplier not blocked
supplier bank safe
invoice not duplicate
PO matched where required
receiving matched where required
inspection matched where required
credit notes applied
replacements accepted or hold remains
refunds tracked where needed
tax evidence acceptable or routed
payment terms known
AP holds resolved or separated
authority conditions satisfied for AP side
audit evidence complete
```

Payment readiness statuses:

```text id="payment_readiness_statuses"
NotReady
PartiallyReady
ReadyForPaymentScheduling
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

> “$1,800 is ready for payment scheduling. $200 remains held pending credit note.”

This is the key.

AP can make part of an invoice ready and part held.

No need to hold everything if only 10 units were broken. This is AP precision, not invoice hostage-taking.

---

## 20. Supplier Balance Base

AP maintains the supplier balance base.

It tracks:

```text id="supplier_balance_base"
open invoices
approved payable amounts
held amounts
credit notes applied
credit notes expected
refunds expected
payments requested
payments made
partial payments
disputes
```

But supplier statement reconciliation belongs to:

```text id="reconciliation_doc"
Global Document 75 — Supplier Statement Reconciliation + Creditor Reporting Engine
```

AP provides base data.

Document 75 reconciles supplier statements.

Boundary rule:

```text id="ap_recon_boundary"
AP owns invoice-level payable truth.
Creditor Reconciliation owns supplier statement-to-AP reconciliation and month-end creditor reporting.
```

Do not merge them.

AP is invoice truth.

Reconciliation is balance proof against supplier claims.

Both matter. Neither wants the other’s job. Probably.

---

## 21. Supplier Statement Boundary

Suppliers send statements.

AP may ingest statement-related documents, but full reconciliation belongs to Document 75.

AP may:

```text id="ap_statement_may"
link statement invoice refs to known invoices
flag unknown invoice claims
provide AP invoice/credit/payment data to reconciliation
block payment of unknown statement-only claims
```

AP must not:

```text id="ap_statement_must_not"
pay from supplier statement alone
overwrite AP balance with supplier statement balance
treat supplier statement as truth
close supplier disputes based only on statement
```

Selene says:

> “Supplier statement includes invoice INV-992, but no invoice, PO, or receiving record exists. I’ll send it to reconciliation as an unknown claim, not payment-ready.”

Supplier statements are claims. Sometimes organized claims. Still claims.

---

## 22. Tax Handoff

AP captures tax evidence but Tax owns tax treatment.

AP extracts:

```text id="ap_tax_extracts"
supplier tax ID
tax invoice status
tax amount
tax code shown
jurisdiction
input tax candidate
non-claimable indicators
missing tax invoice evidence
```

AP sends Tax:

```text id="ap_tax_handoff"
TaxClassificationRequestPacket
InputTaxEvidencePacket
TaxExceptionPacket
```

AP does not invent tax law.

If tax evidence missing:

> “This invoice may not be claimable yet because supplier tax invoice data is missing. I’ll route tax review.”

The invoice can still be payable while tax claimability is under review, depending policy.

Paying supplier and claiming tax are related but not identical. Accounting loves these tiny trapdoors.

---

## 23. Accounting Handoff

AP prepares accounting evidence.

Accounting owns final posting.

AP sends:

```text id="ap_accounting_handoff"
supplier_id
invoice_id
clean_payable_amount
held_amount
credit_notes_applied
tax data
cost center
budget ref
PO ref
receiving ref
inspection ref
asset/inventory/expense hints
payment readiness status
audit ref
```

Possible accounting outcomes:

```text id="accounting_outcomes"
expense
inventory asset
asset capitalization candidate
prepayment
accrual / GRNI
AP liability
tax receivable/payable
credit note reduction
refund receivable
```

AP says:

> “This invoice is matched and ready for accounting handoff.”

Accounting posts.

AP does not post.

We have said this often because apparently software keeps trying.

---

## 24. AP and Cashflow

AP provides upcoming liabilities to Cashflow.

Cashflow receives:

```text id="ap_cashflow_handoff"
clean payable invoices
due dates
payment terms
critical supplier status
early payment discount
held amounts
disputed amounts
expected credit notes
expected refunds
supplier payment priority hints
```

Cashflow decides timing strategy.

AP decides invoice readiness.

Supplier Payment executes approved payment schedule.

Selene says:

> “These invoices are clean payable, but Cashflow will decide whether to pay early, on due date, or delay within terms.”

AP is not cashflow. Cashflow is not AP. Everybody back to their assigned tiny kingdom.

---

## 25. Early Payment Discount and Urgent Supplier Requests

AP records supplier payment requests and discount offers but does not execute payment.

Supplier may request:

```text id="supplier_payment_requests"
early payment
urgent payment
partial payment
early payment discount
payment status update
```

AP checks:

```text id="ap_early_payment_checks"
invoice clean?
credit note pending?
supplier dispute open?
replacement/refund pending?
supplier bank safe?
payment readiness?
```

Then passes to Cashflow / Supplier Payment.

Selene says:

> “Invoice is clean and supplier offers 2% early-payment discount. I’ll pass this to Cashflow and Supplier Payment for timing decision.”

If invoice not clean:

> “Supplier requested urgent payment, but the invoice includes damaged goods still awaiting credit. I will not mark it ready.”

Urgent bad invoice remains bad. Now faster.

---

## 26. Invoice Approval vs Payment Approval

AP approval and payment approval are separate.

AP approval means:

```text id="ap_approval_means"
invoice/amount is valid payable based on evidence
```

Payment approval means:

```text id="payment_approval_means"
money may be scheduled/executed under authority, cashflow, bank safety, and payment policy
```

Selene must separate:

```text id="approval_separation"
AP approved / payment not scheduled
AP partially approved / partial hold
AP blocked / no payment readiness
Payment scheduled / AP already approved
Payment failed / AP still open
```

This is crucial.

Invoice truth is not the same as money movement.

Please tell every old accounting system.

---

## 27. AP Fraud and Risk Detection

AP must detect fraud signals.

Signals:

```text id="ap_fraud_signals"
invoice without PO
duplicate invoice
supplier name similar to approved supplier
supplier bank changed
invoice bank differs from supplier bank record
urgent payment language
first invoice from new supplier
invoice amount just below approval threshold
invoice from unknown email/domain
invoice references no receiving proof
invoice includes rejected/damaged goods
supplier statement claim without invoice
split invoices to avoid threshold
```

Actions:

```text id="ap_fraud_actions"
block payment readiness
apply AP hold
route fraud review
notify Supplier Engine
notify Procurement
notify Payment Engine
require additional verification
```

Selene says:

> “This invoice has fraud signals: no PO, new bank details, and urgent payment wording. I’ve blocked payment readiness.”

Fraud often comes with urgency and nice formatting. Selene should distrust both equally.

---

## 28. AP Automation and Exception-Only Review

Selene auto-handles:

```text id="ap_auto_handles"
invoice capture
invoice extraction
supplier matching
duplicate check
PO match
receiving match
inspection match
credit note matching
routine AP hold
routine clean payable calculation
payment readiness marking
tax evidence handoff
accounting handoff
supplier status response
```

Selene escalates:

```text id="ap_escalates"
unknown supplier
restricted supplier
bank safety risk
duplicate suspected
no PO with no approved exception
receiving mismatch
inspection missing
credit note mismatch
supplier dispute
tax anomaly
large amount
manual override
fraud signal
payment readiness conflict
```

Rule:

```text id="ap_exception_rule"
Routine match = Selene handles.
Mismatch = Selene holds and resolves where policy allows.
Exception = human review.
Payment execution = never AP.
Everything = audited.
```

No human approval for a clean, matched, routine invoice under policy.

No auto-payment for a suspicious invoice because the supplier used “URGENT” in red.

Balance. We are learning.

---

## 29. PH1.D / GPT-5.5 Role

GPT-5.5 may help:

```text id="gpt_ap_allowed"
read messy invoice descriptions
summarize invoice line differences
draft supplier credit note request
draft supplier dispute response
explain AP hold reason
summarize invoice exception
draft internal AP review note
translate supplier invoice comments
prepare AP manager briefing
```

GPT-5.5 must not:

```text id="gpt_ap_forbidden"
approve invoices
mark payment-ready
release holds
close credit obligations
invent receiving proof
invent tax treatment
invent supplier authority
override duplicate detection
execute payments
post ledger
```

GPT can write:

> “The invoice includes ten damaged units.”

GPT cannot decide those units were accepted.

The warehouse proof decides that. GPT can sit down.

---

## 30. Human-Like Selene Interaction

### Clean invoice

> “Invoice INV-884 matches the PO, receiving, and inspection. I’ve marked it ready for payment scheduling.”

### Partial hold

> “The invoice includes 100 units, but only 90 were accepted. I’ve approved 90 and held the value of 10 pending supplier credit.”

### Duplicate

> “This looks like a duplicate invoice from Supplier ABC. I’ve blocked payment readiness and linked it to the original.”

### No PO

> “This invoice has no matching PO or approved recurring bill record. I’ve routed it for Procurement review.”

### Bank risk

> “The invoice is clean, but supplier bank details are under review. I won’t mark it payment-ready yet.”

### Credit note

> “Credit note CN-221 matches the damaged quantity. I’ve applied it and released the AP hold.”

This is what AP should feel like: clear, calm, suspicious when appropriate, and allergic to paying for broken goods.

---

## 31. State Machines

### Supplier Invoice State

```text id="supplier_invoice_state"
Received
ExtractionPending
Extracted
SupplierMatching
SupplierMatched
DuplicateChecking
DuplicateSuspected
MatchingPending
Matched
PartiallyMatched
MismatchDetected
CreditRequired
ReplacementPending
RefundPending
PaymentHold
ExceptionReview
ReadyForPayment
PaymentRequested
Paid
PartiallyPaid
Rejected
Cancelled
Archived
```

### Invoice Line State

```text id="invoice_line_state"
Unmatched
Matched
PartiallyMatched
QuantityMismatch
PriceMismatch
TaxMismatch
NoPO
NoReceiving
InspectionPending
DamagedOrRejected
CreditRequired
Held
ApprovedPayable
Rejected
Closed
```

### AP Hold State

```text id="ap_hold_state"
NoHold
HoldSuggested
HoldApplied
EvidenceRequested
AwaitingCreditNote
AwaitingReplacement
AwaitingRefund
AwaitingApproval
Released
Rejected
Closed
```

### Credit Note State

```text id="credit_note_state"
Expected
Requested
Received
Matched
PartiallyMatched
Mismatch
Applied
Rejected
Closed
Archived
```

### Payment Readiness State

```text id="payment_readiness_state"
NotReady
PartiallyReady
ReadyForPaymentScheduling
BlockedSupplierRisk
BlockedBankRisk
BlockedDuplicate
BlockedNoPO
BlockedReceivingMismatch
BlockedInspectionPending
BlockedCreditPending
BlockedTaxReview
BlockedAuthority
Closed
```

### Invoice Dispute State

```text id="invoice_dispute_state"
Detected
EvidenceAttached
SupplierNotified
AwaitingSupplierResponse
SupplierResponded
AcceptedBySupplier
RejectedBySupplier
InternalReview
Escalated
Resolved
Closed
Archived
```

---

## 32. Reason Codes

```text id="ap_reason_codes"
SUPPLIER_INVOICE_RECEIVED
SUPPLIER_INVOICE_EXTRACTED
SUPPLIER_MATCHED
SUPPLIER_NOT_FOUND
SUPPLIER_RESTRICTED
SUPPLIER_BLOCKED
SUPPLIER_BANK_RISK
DUPLICATE_INVOICE_SUSPECTED
NO_MATCHING_PO
PO_MATCHED
RECEIVING_MATCHED
INSPECTION_MATCHED
QUANTITY_MISMATCH
PRICE_MISMATCH
TAX_REVIEW_REQUIRED
CREDIT_NOTE_REQUIRED
CREDIT_NOTE_REQUESTED
CREDIT_NOTE_MATCHED
CREDIT_NOTE_MISMATCH
REPLACEMENT_PENDING
REPLACEMENT_ACCEPTED
REFUND_PENDING
REFUND_MATCHED
AP_HOLD_APPLIED
AP_HOLD_RELEASED
PAYABLE_AMOUNT_CONFIRMED
DISPUTED_AMOUNT_HELD
PAYMENT_READINESS_BLOCKED
READY_FOR_PAYMENT_SCHEDULING
NON_PO_INVOICE_ALLOWED
NON_PO_INVOICE_BLOCKED
AP_FRAUD_SIGNAL
ACCOUNTING_HANDOFF_READY
TAX_HANDOFF_READY
```

---

## 33. Required Simulations

```text id="ap_simulations"
invoice captured from email
invoice captured from supplier portal
invoice submitted by Supplier Selene
unknown supplier invoice blocked
duplicate invoice detected
clean PO invoice matched
PO invoice matched to receiving
four-way match clean invoice
invoice includes damaged quantity
invoice includes short quantity
AP hold created from receiving
credit note requested
credit note matched and hold released
credit note mismatch remains held
replacement accepted and hold released
refund expected after paid invoice
non-PO recurring utility invoice accepted
non-PO unauthorized invoice blocked
supplier bank risk blocks payment readiness
tax evidence missing routes Tax review
supplier early payment request recorded
early payment discount passed to Cashflow
supplier statement unknown invoice sent to Reconciliation
AP accounting handoff created
invoice fraud signal detected
```

---

## 34. Integration Map

```text id="ap_integration_map"
PH1.CREDITORS / ACCOUNTS_PAYABLE
↔ PH1.SUPPLIER
↔ PH1.SUPPLIER.BANK_TRUST
↔ PH1.PROCUREMENT / PH1.PROC.ORDER
↔ PH1.PROC.RECEIVE
↔ PH1.INVENTORY
↔ PH1.PRODUCT
↔ PH1.SUPPLIER_PAYMENT
↔ PH1.CREDITORS.RECON
↔ PH1.ACCOUNTING
↔ PH1.TAX
↔ PH1.TAX.OPTIMIZE
↔ PH1.CASHFLOW
↔ PH1.BUDGET
↔ PH1.BANKREC / TREASURY
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

## 35. Required Logical Packets

```text id="ap_packets"
SupplierInvoiceIntakePacket
SupplierInvoiceExtractionPacket
SupplierInvoicePacket
SupplierInvoiceLinePacket
SupplierMatchPacket
InvoiceDuplicateCheckPacket
InvoiceMatchingPacket
TwoWayMatchPacket
ThreeWayMatchPacket
FourWayMatchPacket
RecurringBillMatchPacket
PayableAmountPacket
DisputedAmountPacket
APHoldPacket
CreditNotePacket
CreditNoteMatchPacket
ReplacementTrackingPacket
RefundTrackingPacket
InvoiceDisputePacket
PaymentReadinessPacket
TaxHandoffPacket
AccountingHandoffPacket
SupplierBalanceBasePacket
APFraudSignalPacket
APAudiEvidencePacket
```

Logical only. Codex maps later. No packet structs. The schema goblin remains in its little cave.

---

## 36. What Codex Must Not Do

```text id="codex_no_ap"
Do not merge AP with Supplier Payment.
Do not let AP execute bank payments.
Do not let AP post ledger directly.
Do not let AP create receiving truth.
Do not let supplier invoice create stock truth.
Do not pay invoice without required matching proof.
Do not pay disputed quantities.
Do not ignore credit notes owed.
Do not treat supplier statements as AP truth.
Do not merge Supplier Statement Reconciliation into AP.
Do not let GPT-5.5 approve invoices or payment readiness.
Do not require pointless human approval for clean matched routine invoices.
Do not implement from this document alone.
```

---

## 37. Final Architecture Sentence

Selene AP / Creditors Engine is the supplier invoice truth layer that captures and extracts invoices, matches suppliers, detects duplicates and fraud signals, performs two-way, three-way, four-way, recurring, and exception matching against purchase orders, receiving, inspection, contracts, credit notes, replacements, refunds, supplier obligations, tax evidence, and bank safety, calculates clean payable and disputed held amounts, prepares payment readiness for Supplier Payment, provides accounting and tax handoffs, supports partial holds and partial readiness, and uses GPT-5.5 for explanation and supplier communication while deterministic Selene proof, policy, Supplier, Procurement, Receiving, Payment, Accounting, Tax, Authority, and Audit preserve financial control.

Simple version:

```text id="ap_simple"
Supplier sends invoice.
Selene reads it.
Selene checks supplier.
Selene checks duplicate risk.
Selene checks PO.
Selene checks receiving.
Selene checks inspection.
Selene checks credits, replacements, and refunds.
Selene checks bank safety.
Selene approves only clean payable amount.
Selene holds disputed value.
Selene prepares payment readiness.
Payment Engine pays later.
Accounting posts later.
Humans review only exceptions.
Everything is audited.
```

That is Global Document 73 — AP / Creditors Engine. AP is now not an inbox full of supplier demands. It is the invoice-control brain that politely tells suppliers, “Yes, we owe this part, no, we are not paying for the broken part, and please stop sending the same invoice twice with a new filename.”

---

## 38. 81E B2B AP Hold, Reversal + Credit Note Handoff

AP must respect B2B refund, reversal, clawback, reserve, credit-note, provider-fault, return courier, and disputed-commission effects from 81E and Document 78 when calculating payable readiness.

AP holds must recognize unresolved return courier allocation, provider fault, credit notes owed, settlement holds, disputed benefit funding, reserve release conditions, and commission/payout clawbacks before sending payment readiness to Supplier Payment.

---

## 39. 81F-81G Promotion Liability + Reversal Handoff

Promotion liabilities, future credits, loyalty credits, cashback, provider-funded benefits, B2B benefit reversals, and pricing/promotion refund reversals must be visible to AP and Accounting where supplier or provider obligations are impacted.

AP must not treat pricing/promotion/B2B clawbacks, credits, disputed reversals, or benefit obligations from 81E, 81F, or 81G as clean payables without reconciliation evidence.

---

## 40. Commerce Stack 83 AP Credit Note + Provider Hold Handoff

AP/Creditors must respect supplier/provider credit notes, return/refund adjustments, reserve usage, provider payout holds, return courier allocations, refund reversals, customer benefit reversals, settlement holds, and supplier/provider obligation changes created by Document 83.

AP must not clear supplier/provider balances where Document 83, 81E, or 81G indicates unresolved dispute evidence, inspection outcome, credit-note requirement, clawback, settlement hold, or reversal.
