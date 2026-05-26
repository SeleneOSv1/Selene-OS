# Selene Accounts Payable Addendum — Critical Payment Timing + Goods Receiving + Supplier Resolution + 4-Way Match Law

```text
DOCUMENT TYPE:
MASTER DESIGN ADDENDUM / ACCOUNTS PAYABLE + PROCUREMENT RECEIVING + PAYMENT TIMING CONTROL

STATUS:
MASTER_DESIGN
NOT_RUNTIME_IMPLEMENTATION
PENDING_GRAND_ARCHITECTURE_RECONCILIATION

APPLIES TO:
Document 3 — Selene Accounts Payable + Supplier Bills + Installments + Scheduled Payments Master Design

PURPOSE:
Strengthen Document 3 with critical payment timing rules, no-late loan/lease/payment obligations, goods receiving, inspection/acceptance, supplier dispute resolution, 4-way purchase-to-pay matching, accepted-goods-only payment release, cashflow-aware payment control, and AP/Accounting/Procurement/Receiving owner boundaries.
```

## 0. Authority and Scope

AGENTS.md controls execution.

This is a docs-only master design addendum.

No runtime code was changed.

This document does not authorize implementation.

This document is Finance/Accounting Design Batch 2. It references a future `PH1.PROC.RECEIVE` engine boundary but does not create that standalone engine or any implementation.

Current repo truth does not prove runtime AP, Procurement, Receiving, Inspection, Supplier Resolution, Banking, Payment Provider, Accounting, Cashflow, or 4-way matching engines. This addendum is future design pending Grand Architecture Reconciliation.

## 1. Master Addendum Law

Selene must not pay suppliers just because an invoice exists.

Selene must not pay late on critical obligations just because humans forgot approvals, bank cutoffs, weekends, or payment clearing times.

The upgraded AP law is:

```text
No PO = block invoice unless an approved non-PO exception exists.

No receiving proof = hold payment.

No inspection / acceptance proof = hold or limit payment.

No invoice match = supplier dispute or AP exception.

Accepted goods / accepted services = payable.

Damaged, missing, rejected, wrong, or disputed goods = payment hold, credit note, refund, replacement, discount, cancellation, or authorized override.

Critical obligations such as bank loans, leases, rent, tax, payroll-related remittances, insurance, and legally binding installment payments must be scheduled early enough for cleared funds to arrive before the due date.
```

## 2. Critical Payment Timing Law

For loans, leases, bank obligations, rent, tax, insurance, payroll-related remittances, and other critical payments, the due date is not the payment-send date.

Selene must calculate the latest safe start date.

```text
payment_due_date
minus bank_processing_days
minus receiver_clearing_days
minus weekend/public_holiday_adjustment
minus internal_approval_lead_time
minus payment_provider_risk_buffer
= latest_safe_payment_start_date
```

Selene must consider:

```text
bank processing time
payment rail cutoff
weekends
public holidays
country banking calendars
currency / FX settlement time
receiver clearing time
internal approval time
multi-authority approval time
cash reserve rule
payment provider failure risk
late fee risk
default risk
loan covenant risk
lease breach risk
credit rating risk
```

Example:

```text
Lease payment due: Friday
Bank transfer clearing: 2 business days
Approval lead time: 1 business day
Latest safe start: Tuesday
```

Selene says:

```text
The lease payment is due Friday, but funds need two business days to clear. I'll start the approval and payment flow today so it reaches the receiver on time.
```

## 3. Critical Obligation Priority

Critical obligations include:

```text
bank loans
car loans
equipment finance
property loans
leases
rent
tax payments
payroll payments
superannuation / CPF / pension remittances
insurance premiums
utilities that would stop operations
court/legal obligations
regulated/statutory payments
critical supplier obligations
```

Selene must flag:

```text
late_fee_risk
default_risk
covenant_breach_risk
service_cutoff_risk
legal_penalty_risk
credit_rating_risk
cash_reserve_breach_risk
```

If cash is short, Selene must not immediately delay critical payments.

Selene must activate the cashflow protocol:

1. Chase receivables.
2. Accelerate sales/cash generation where possible.
3. Reprioritize flexible outgoing payments.
4. Request emergency approval if a critical payment breaches reserve.
5. Recommend partial/delayed payment only where lawful and contractually allowed.

## 4. 4-Stage Purchase Order Control Flow

Selene must follow a governed purchase-to-pay flow:

1. Purchase Order
2. Goods Receipt
3. Inspection / Acceptance
4. Invoice Match + Payment Release

This is mandatory for stocked goods, inventory, assets, materials, equipment, and any supplier category where company policy requires receiving proof.

### Stage 1 — Purchase Order

Every controlled purchase starts with an approved purchase order unless a company-approved non-PO exception exists.

Selene checks:

```text
requester identity
requester purchase limit
budget availability
supplier approval status
supplier bank verification
item/category allowed
delivery location
cost center
required approval path
cashflow impact
tax treatment candidate
asset/inventory/expense classification candidate
```

The purchase order must include:

```text
purchase_order_id
requester_id
purchase_owner_id
supplier_id
items
item_description
quantity_ordered
unit_price
total_price
currency
tax_code_candidate
delivery_location
required_delivery_date
budget_ref
cost_center_ref
project_ref
appointed_receiver_id
appointed_quantity_checker_id
appointed_damage_quality_checker_id
inspection_required
approval_status
audit_ref
```

Selene may proactively remind:

```text
You usually order packaging materials around this time. Stock looks low. Do you want me to prepare the purchase order?
```

### Stage 2 — Goods Receipt

When goods arrive, the appointed receiver records what physically arrived.

Goods receipt fields:

```text
goods_receipt_id
purchase_order_id
supplier_id
delivery_note_ref
received_by_user_id
received_at
received_location
quantity_delivered
items_delivered
missing_items
wrong_items
visible_damage
photos_or_evidence_refs
delivery_vehicle_or_tracking_ref
receiver_comment
audit_ref
```

Rule:

```text
No receiving proof = no payment release.
```

Example:

```text
PO ordered: 100 pieces
Delivered: 95 pieces
Receiver records: 95 received, 5 short
```

### Stage 3 — Inspection / Acceptance

The appointed inspector checks whether delivered goods are usable and accepted.

Inspection fields:

```text
inspection_id
goods_receipt_id
purchase_order_id
inspected_by_user_id
inspected_at
accepted_quantity
damaged_quantity
rejected_quantity
wrong_item_quantity
quality_issue_description
photo_evidence_refs
requires_supplier_resolution
recommended_resolution
inspection_status
audit_ref
```

Inspection statuses:

```text
AcceptedFull
AcceptedPartial
RejectedFull
Damaged
WrongItems
MissingItems
QualityHold
PendingReview
```

Example:

```text
Delivered: 95
Accepted: 90
Damaged: 5
Missing from PO: 5
```

Payment rule:

```text
Only accepted goods / accepted services become payable.

Damaged, missing, rejected, or disputed goods stay on hold unless an authorized override passes.
```

### Stage 4 — Invoice Match + Payment Release

Before AP pays, Selene performs a 4-way match:

```text
Purchase Order
+ Goods Receipt
+ Inspection / Accepted Quantity
+ Supplier Invoice
```

4-way match fields:

```text
four_way_match_id
purchase_order_id
goods_receipt_id
inspection_id
supplier_invoice_id
ordered_quantity
delivered_quantity
accepted_quantity
rejected_quantity
damaged_quantity
missing_quantity
invoice_quantity
ordered_unit_price
invoice_unit_price
accepted_payable_amount
disputed_amount
tax_difference
variance_amount
variance_percent
match_status
payment_release_status
audit_ref
```

Good match:

```text
PO: 100 items for $2,000
Received: 100
Accepted: 100
Invoice: 100 items for $2,000
Result: approved for payment
```

Failed match:

```text
PO: 100 items for $2,000
Received: 95
Accepted: 90
Invoice: 100 items for $2,000
Result: payment blocked or reduced
```

Selene tells AP:

```text
Pay accepted quantity only.
Hold disputed quantity.
Request credit, replacement, refund, discount, or cancellation from supplier.
Audit the full chain.
```

## 5. Supplier Resolution Cycle

If goods are short, damaged, wrong, late, overbilled, or rejected, Selene opens a supplier resolution case.

Resolution reasons:

```text
missing_items
damaged_items
wrong_items
quality_failure
late_delivery
invoice_quantity_exceeds_accepted_quantity
invoice_price_exceeds_PO
tax_mismatch
duplicate_invoice
goods_not_received
service_not_completed
```

Resolution options:

```text
replacement
credit_note
refund
discount
cancel_remaining_order
partial_payment
full_payment_hold
supplier_dispute
supplier_risk_escalation
authorized_override
```

Resolution case fields:

```text
supplier_resolution_case_id
supplier_id
purchase_order_id
goods_receipt_id
inspection_id
supplier_invoice_id
issue_type
accepted_quantity
disputed_quantity
disputed_amount
evidence_refs
requested_resolution
supplier_response_status
internal_owner_id
status
audit_ref
```

Resolution statuses:

```text
Opened
SupplierNotified
AwaitingSupplierResponse
ReplacementRequested
CreditNoteRequested
RefundRequested
DiscountRequested
PartialPaymentApproved
Resolved
Rejected
Escalated
Closed
```

Selene may draft supplier communication through PH1.WRITE.

Delivery must use PH1.BCAST / PH1.DELIVERY or an approved supplier connector.

## 6. Payment Release Rules

AP may release payment only for:

```text
accepted goods
accepted services
approved milestones
approved partial receipt
authorized override
contractually payable deposits
approved progress payments
```

AP must hold payment for:

```text
missing goods
damaged goods
rejected goods
wrong goods
uninspected goods where inspection is required
goods not received
unapproved service completion
contractor overrun
invoice quantity above accepted quantity
invoice price above approved tolerance
supplier bank change under review
```

Partial payment formula:

```text
payable_amount =
accepted_quantity * approved_unit_price
+ approved_tax
- credit_notes
- discounts
- prior_payments
```

If tax depends on final accepted quantity, Tax owner must recalculate.

## 7. Accounting Treatment

Accounting must only post final truth once receiving/inspection status is known.

Accepted goods:

```text
Debit: Inventory / Expense / Asset
Debit: GST/VAT Receivable if claimable
Credit: Accounts Payable
```

Damaged/rejected goods:

```text
No final AP liability for rejected portion unless policy/contract says otherwise.
Hold in dispute.
Record memo/evidence only until resolution.
```

Credit note:

```text
Debit: Accounts Payable
Credit: Inventory / Expense / Tax Receivable adjustment
```

Refund:

```text
Debit: Bank
Credit: Accounts Payable / Supplier Receivable / Expense Recovery
```

Cancelled order:

```text
Close PO.
Remove scheduled AP payment.
Audit cancellation.
Reverse or adjust any provisional entries if posted.
```

Accounting owns postings.

Receiving owns physical receipt proof.

Inspection owns acceptance proof.

AP owns bill/payment status.

## 8. Goods Receiving + Supplier Resolution Engine Boundary

This addendum introduces the future module:

```text
PH1.PROC.RECEIVE
Goods Receiving + Inspection + Supplier Resolution Engine
```

This is a future design target only. It is not created or implemented by this document.

PH1.PROC.RECEIVE owns:

```text
goods receipt
quantity received
inspection evidence
accepted quantity
damaged quantity
rejected quantity
wrong items
missing items
supplier resolution case
photos/evidence refs
receiver/inspector accountability
```

PH1.PROC.RECEIVE connects:

```text
Procurement -> Receiving -> Inspection -> Inventory/Assets -> AP -> Accounting -> Finance -> Banking
```

AP consumes receiving/inspection evidence.

AP does not create receiving truth.

Inventory/Assets consume accepted goods evidence.

Inventory/Assets do not approve supplier payment.

Accounting consumes accepted/credit/refund evidence.

Accounting does not inspect goods.

## 9. Responsible People Assignment

When a PO is approved, Selene must assign or confirm responsible people:

```text
purchase_owner_id
receiving_officer_id
quantity_checker_id
damage_quality_checker_id
approval_owner_id
budget_owner_id
AP_owner_id
```

Example:

```text
Purchase Owner: Tom
Receiving Officer: Sarah
Quantity Checker: Ahmed
Damage/Quality Checker: Priya
AP Owner: Finance Team
```

If no receiver/inspector is assigned for goods requiring proof:

```text
Selene blocks PO finalization or asks for a responsible person.
```

## 10. GPT-5.5 Role

PH1.D / GPT-5.5 may help:

```text
draft supplier dispute messages
summarize receiving issue
explain mismatch to AP
help classify dispute type
translate supplier communication
summarize photo/document evidence text
draft internal approval summary
```

PH1.D / GPT-5.5 must not:

```text
confirm goods were received
approve damaged goods
release payment
decide refund truth
decide credit note truth
override inspection
override PO match
override cash reserve
execute bank transfer
post accounting entries
```

Physical receipt and acceptance require deterministic evidence and responsible users.

## 11. Access / Authority

Protected actions include:

```text
override accepted-goods-only payment rule
approve payment for disputed goods
approve payment without receipt
approve payment without inspection
approve supplier credit settlement
approve emergency critical payment below cash reserve
change receiver/inspector after delivery
approve PO variance above tolerance
release held supplier payment
```

Authority may depend on:

```text
amount
supplier risk
purchase category
asset/inventory/expense type
department
country
budget status
cash reserve impact
dispute value
urgency
legal/contract risk
```

Step-up may be required for:

```text
payment override
cash reserve breach
high-value disputed payment
supplier bank/payment change
emergency loan/lease/tax payment approval
```

## 12. Critical Payment Failure Handling

Critical payments cannot be allowed to become late without escalation.

If Selene detects risk:

```text
approval not complete
bank cutoff approaching
cash reserve breach
payment provider unavailable
public holiday delay
insufficient cash
missing invoice/contract evidence
```

Selene must:

```text
escalate immediately
show deadline and consequence
propose payment options
request emergency approval if needed
prioritize collections/cashflow actions
document risk
audit
```

Example:

```text
The vehicle loan payment must clear by Friday. Because tomorrow is a public holiday and the bank rail takes two business days, approval is needed today to avoid late payment.
```

## 13. Additions To Document 3 "What Must Not Happen"

Add:

```text
no critical loan/lease/tax/rent/payment scheduled too late for funds to clear by due date
no supplier invoice paid just because the invoice says goods were supplied
no payment without goods receipt where receipt is required
no payment without inspection where inspection is required
no payment for damaged/missing/rejected goods unless authorized override passes
no AP-created receiving truth
no Inventory-created supplier payment truth
no Accounting-created inspection truth
no GPT-5.5 confirmation of physical receipt
no supplier dispute ignored
no credit note/refund/replacement resolution lost
no accepted quantity overwritten by invoice quantity
no PO/invoice/receipt/inspection mismatch hidden
no payment of disputed supplier amount without explicit authority
no critical obligation missed because approval started too late
```

## 14. Required Logical Packets

Future logical packets:

```text
PurchaseOrderControlPacket
ReceivingOfficerAssignmentPacket
GoodsReceiptPacket
InspectionAcceptancePacket
FourWayMatchPacket
AcceptedGoodsPaymentReleasePacket
SupplierResolutionCasePacket
SupplierDisputeEvidencePacket
SupplierCreditRequestPacket
SupplierReplacementRequestPacket
SupplierRefundRequestPacket
CriticalPaymentTimingPacket
PaymentClearingDeadlinePacket
EmergencyReserveOverrideRequestPacket
CriticalPaymentRiskAlertPacket
```

Codex must later map these to repo equivalents.

Do not claim they currently exist.

## 15. Example — PO 100 Items, Delivery 90 Good / 5 Damaged / 5 Missing

PO:

```text
100 pieces
$2,000 total
```

Delivery:

```text
95 pieces arrived
```

Inspection:

```text
90 accepted
5 damaged
5 missing
```

Invoice:

```text
100 pieces
$2,000 total
```

Selene result:

```text
accepted_quantity: 90
held_quantity: 10
payment_release_status: partial_or_hold
supplier_resolution_case: opened
requested_resolution:
  - replacement for 5 missing
  - credit/refund/replacement for 5 damaged
AP payment:
  - pay only accepted portion if policy allows
  - or hold full invoice until dispute resolved
Accounting:
  - post only accepted goods if policy allows partial posting
  - adjust later with credit note/refund/replacement
Audit:
  - PO, receipt, inspection, invoice, photos, dispute, approval refs linked
```

Selene says:

```text
The invoice is for 100 pieces, but only 90 were accepted. I've held the disputed portion and opened a supplier resolution case for the 5 damaged and 5 missing items. AP can only release payment for the accepted quantity unless an authorized override is approved.
```

## 16. Example — Loan Payment Due

```text
Loan payment due: 30 June
Receiver clearing time: 2 business days
Bank cutoff: 4 PM
Approvals required: CFO + CEO
Latest safe approval date: 26 June
Latest safe bank send date: 27 June
```

Selene says:

```text
The loan payment is due on 30 June, but funds need two business days to clear and both CFO and CEO approval are required. I'll start the approval flow on 26 June so the payment is not late.
```

If approval is not completed:

```text
The loan payment is now at risk of being late. CEO and CFO approval is still missing. I'm escalating this as a critical payment risk.
```

## 17. Example — Wrong Goods Supplier Resolution

PO:

```text
10 replacement tyres
$3,000 total
```

Delivery:

```text
10 tyres arrived
```

Inspection:

```text
8 accepted
2 wrong model
```

Invoice:

```text
10 tyres
$3,000
```

Selene result:

```text
accepted_quantity: 8
wrong_item_quantity: 2
supplier_resolution_case: opened
requested_resolution: replacement or credit note for 2 wrong tyres
payment_release_status: partial_or_hold
```

Selene says:

```text
The delivery included 2 tyres that do not match the order. I've held that portion and opened a supplier resolution case. AP can release payment only for the 8 accepted tyres unless an authorized override is approved.
```

## 18. Future Simulation Targets

Add these future simulations:

```text
SIM_AP_012_critical_lease_payment_scheduled_for_cleared_due_date
SIM_AP_013_loan_payment_approval_starts_before_bank_cutoff
SIM_PROC_RECEIVE_001_po_receiver_assignment
SIM_PROC_RECEIVE_002_goods_receipt_partial_delivery
SIM_PROC_RECEIVE_003_inspection_accepts_partial_rejects_damaged
SIM_AP_014_four_way_match_blocks_overbilled_invoice
SIM_AP_015_accepted_goods_only_partial_payment
SIM_AP_016_supplier_credit_note_resolution
SIM_AP_017_supplier_replacement_resolution
SIM_AP_018_payment_without_receipt_blocked
SIM_AP_019_payment_without_inspection_blocked
SIM_AP_020_emergency_cash_reserve_override_for_critical_payment
SIM_AP_021_wrong_goods_supplier_resolution
SIM_AP_022_critical_payment_approval_timeout_escalation
```

## 19. Final Addendum Architecture Sentence

Selene Accounts Payable must not pay blindly and must not pay critical obligations late: every controlled purchase must pass purchase order, goods receipt, inspection/acceptance, and invoice match before payment release, while loans, leases, rent, tax, payroll-related remittances, and other critical obligations must be scheduled early enough for cleared funds to arrive before due date, with cashflow checks, authority gates, supplier resolution, accepted-goods-only payment logic, accounting handoff, banking proof, and full audit.
